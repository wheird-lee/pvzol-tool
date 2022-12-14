use std::str::FromStr;

use clap::{Subcommand};
use lib::{game::sys::{Quality, QualityUpType}, Client, Result};

macro_rules! warn_ignored {
    ($lit:literal) => {
        eprintln!("warning: option `{}` is ignored.", $lit)
    };
}

#[derive(Subcommand)]
#[non_exhaustive]
pub(crate) enum Command {

    /// 刷新品质
    QualityUp {
        /// 目标品质
        #[clap(long, value_parser)]
        until: Option<Quality>,

        /// 使用魔神刷新书
        #[clap(short, long, action)]
        moshen: bool,

        /// 植物Id
        #[clap(value_parser, required = true)]
        plant_id: Vec<f64>,

    },

    /// 提升技能等级
    SkillUp {
        /// 植物Id
        #[clap(value_parser)]
        plant_id: f64,

        /// 技能Id
        #[clap(value_parser)]
        skill_id: f64,

        #[clap(long = "up", value_parser, value_name = "LEVEL")]
        up_level: Option<u32>,
    },

    /// 开宝箱
    Open {
        /// 箱子ID
        #[clap(value_parser)]
        box_id: f64,

        /// 需要开启的数量
        #[clap(value_parser)]
        amount: Option<u32>,
    },

    /// 自动挑战洞穴/副本, 并领取奖励
    Challenge {

        /// 世界副本
        #[clap(short = 'f', long = "fuben", action)]
        is_fuben: bool,

        /// 宝石副本
        #[clap(short = 's', long = "stone", action)]
        is_stone: bool,

        /// 关卡Id
        #[clap(value_parser)]
        id: f64,

        /// 出战植物Id
        #[clap(value_parser)]
        plant_ids: Vec<f64>,

    },

    #[cfg(feature = "hack")]
    #[clap(subcommand)]
    Hack(HackCommand),
}

#[cfg(feature = "hack")]
#[derive(Subcommand)]
pub(crate) enum HackCommand {
    Duty {
        #[clap(value_parser)]
        duty_ids: Vec<f64>,
    },

    Fuben {
        #[clap(value_parser)]
        fuben_id: f64,

        /// reset but not get fuben reward
        #[clap(short, long, action)]
        reset: bool,

    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderBy {
    Name,
    Level,
}

impl Command {
    pub async fn invoke_on(self, client: &Client, repeat: Option<usize>) -> Result<()> {
        use Command::*;

        let repeat_times = repeat.unwrap_or(1) as usize;

        match self {
            QualityUp {
                plant_id: plant_ids,
                until,
                moshen,
            } => {
                let (until, quality_up_type) = match moshen {
                    true => {
                        if until.is_some() {
                            eprintln!("warning: 使用魔神刷新书, 参数`--until`已忽略");
                        }
                        (Some(Quality::魔神), QualityUpType::Moshen)
                    },
                    false => (until, QualityUpType::General),
                };
                let until_fn: Box<dyn Fn(usize,Quality)->bool> = match until {
                    Some(to_quality) => {
                        if repeat.is_some() {
                            warn_ignored!("repeat")
                        }
                        Box::new(move |_, q| q == to_quality)
                    },
                    None => Box::new(move |i,_| i >= repeat_times),
                };
                for plant_id in plant_ids {
                    client.quality_up_to(quality_up_type, plant_id, &until_fn).await?;
                }
            },
            SkillUp {
                plant_id,
                skill_id,
                up_level
            } => {
                let until: Box<dyn Fn(usize,u32)->bool> = match up_level {
                    Some(up) => {
                        if repeat.is_some() {
                            warn_ignored!("repeat");
                        }
                        Box::new(move |_, uped| uped == up)
                    },
                    None => Box::new(move |i,_| i >= repeat_times),
                };
                client.skill_up_to(plant_id, skill_id, until).await?;
            },
            Open { box_id, amount } => {
                let amount = amount.unwrap_or(1);
                if amount == 0 {
                    return Err("单次开启数量必须大于1且小于11".into());
                } else if amount > 10 {
                    return Err("单次开启数量必须小于11, 如果想开启多个, 请使用`--repeat`参数".into());
                }
                client.open_box_repeat(box_id, amount, repeat_times).await?;
            },
            Challenge {is_fuben, is_stone, id: fuben_id, plant_ids } => {
                if is_fuben {
                    client.challenge_fuben_repeat(fuben_id, plant_ids, repeat_times).await?;
                } else if is_stone {
                    // compile_error!("unimplement");
                    return Err("该功能未完成".into());
                } else {
                    return Err("未给定挑战类型.(公洞/个洞/按洞/副本/...)".into());
                }
            },
            #[cfg(feature = "hack")]
            Hack(hack) => {
                hack.invoke_on(client, repeat).await?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "hack")]
impl HackCommand {
    pub async fn invoke_on(self, client: &Client, repeat: Option<usize>) -> Result<()> {
        let repeat_times = repeat.unwrap_or(1);
        match self {
            HackCommand::Duty { duty_ids } => {
                if repeat.is_some() {
                    warn_ignored!("--repeat");
                }
                client.get_duty_rewards(duty_ids.into_iter(), 3.0).await?;
            },
            HackCommand::Fuben { fuben_id, reset } => {
                if reset {
                    if repeat.is_some() {
                        warn_ignored!("--repeat");
                    }
                    client.reset_fuben_reward(fuben_id).await?;
                } else {
                    client.reset_and_get_fuben_reward(fuben_id, repeat_times).await?;
                }
            },
        }
        Ok(())
    }
}

impl std::fmt::Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(&self, f)
    }
}

impl FromStr for OrderBy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use OrderBy::*;

        if s.len() > 5 || !s.is_ascii() {
            return Err(format!("fail to parse `{}` as OrderBy", s));
        }

        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "n" | "name" => Ok(Name),
            "l" | "level" => Ok(Level),
            _ => Err(format!("fail to parse `{}` as OrderBy", s)),
        }
    }
}
