//! ## 支持的功能:
//!   1. 使用技能书或品刷
//!   2. 开箱子
//!   3. 宝石合成
//!   4. 自动卡bug刷材料
//!      (1) 任务bug
//!      (2) 副本bug (完成度&勋章)
//! 
//! ## 未来可能支持的功能:
//!   1. 带级
//!   2. 自动合成、滚包(需要准备好材料)
//!  

use std::path::PathBuf;

use clap::{AppSettings, ArgGroup, Parser};
use command::Command;
use lib::{Client, AccountInfo, Result};

mod command;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(group(
    ArgGroup::new("user-config")
        .required(true)
        .args(&["user", "config",]),
))]
struct Cli {
    /// 配置文件名 (如果给定该选项, 则会从当前路径查找配置文件)
    #[clap(short, long, value_parser)]
    user: Option<String>,

    /// 配置文件路径
    #[clap(short, long, value_parser, value_name = "FILE_NAME")]
    config: Option<PathBuf>,

    /// 重复执行次数 (仅对某些命令有效)
    #[clap(long = "repeat", value_name = "TIMES", value_parser = clap::value_parser!(u64).range(1..))]
    repeat_times: Option<u64>,

    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();

    let config_file = cli.config
        .or(cli.user.map(Into::into))
        .expect("必须给定用户信息");

    if !config_file.exists() {
        panic!("找不到给定的配置文件\"{:?}\"", config_file.as_os_str());
    }

    let client = Client::builder()
        .account(AccountInfo::from_file(config_file).await?)
        .build()?;

    cli.command.invoke_on(&client, cli.repeat_times.map(|n| n as usize)).await?;

    Ok(())
}
