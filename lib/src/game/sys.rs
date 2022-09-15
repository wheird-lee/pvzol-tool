
use std::{str::FromStr};

use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::game::*;

pub struct SysInfo {
    pub(crate) organisms: Vec<Organism>,
    pub(crate) tools: Vec<Tool>,
}

pub(crate) static SYS_INFO: OnceCell<SysInfo> = OnceCell::new();

pub fn get_sys_organisms() -> Result<&'static[Organism]> {
    if let Some(sys_info) = SYS_INFO.get() {
        let organisms = sys_info.organisms.as_slice();
        unsafe {
            // "almost safe" operation, because sys_info's lifetime is almost equal to the whole app.
            return Ok(&*(organisms as *const [Organism]));
        }
    }
    Err(ErrorKind::DataNotInitialized)
}

pub async fn get_sys_tools() -> Result<&'static[Tool]> {
    if let Some(sys_info) = SYS_INFO.get() {
        let tools = sys_info.tools.as_slice();
        unsafe {
            // "almost safe" operation, because sys_info's lifetime is almost equal to the whole app.
            return Ok(&*(tools as *const [Tool]));
        }
    }
    Err(ErrorKind::DataNotInitialized)
}

impl SysInfo {
    
}

#[derive(Debug, Deserialize)]
pub struct Organism {
    pub id: Id,
    pub name: String,

    #[serde(rename = "type")]
    pub organism_type: u32,

    /// 属性
    pub attribute: String,

    pub height: u32,
    pub width: u32,

    #[serde(rename = "img_id")]
    pub image_id: Id,

    pub evolutions: Vec<Evolution>,
}

#[derive(Debug, Deserialize)]
pub struct Evolution {
    pub id: Id,
    pub grade: Grade,
    pub target: Id,
    pub tool_id: Id,
    pub money: usize,
}

#[derive(Debug, Deserialize)]
pub struct Tool {
    pub tool_id: Id,
    pub name: String,
    pub image_id: Id,
    pub tool_type: u32,
    pub type_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Quality {
    劣质,
    普通,
    优秀,
    精良,
    极品,
    史诗,
    传说,
    神器,
    魔王,
    战神,
    至尊,
    魔神,
    耀世,
    不朽,
    永恒,
    太上,
    无极,
    混沌,
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(&self, f)
    }
}

impl FromStr for Quality {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "劣质" => Quality::劣质,
            "普通" => Quality::普通,
            "优秀" => Quality::优秀,
            "精良" => Quality::精良,
            "极品" => Quality::极品,
            "史诗" => Quality::史诗,
            "传说" => Quality::传说,
            "神器" => Quality::神器,
            "魔王" => Quality::魔王,
            "战神" => Quality::战神,
            "至尊" => Quality::至尊,
            "魔神" => Quality::魔神,
            "耀世" => Quality::耀世,
            "不朽" => Quality::不朽,
            "永恒" => Quality::永恒,
            "太上" => Quality::太上,
            "无极" => Quality::无极,
            "混沌" => Quality::混沌,
            _ => Err(format!("无法将`{}`解析为品质", s))?
        })
    }
}

pub enum ChallengeType {
    /// 副本
    Fuben,

    /// 宝石副本
    Stone,

}

impl ChallengeType {
    pub fn get_amf_target(challenge_type: &ChallengeType) -> &'static str {
        use ChallengeType::*;

        match challenge_type {
            Fuben => "api.fuben.challenge",
            Stone => "api.stone.challenge",            
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityUpType {
    /// 使用品质刷新书
    General,
    
    /// 使用魔神刷新书
    Moshen,
}

pub trait GetSysInfo {
    // http://s36.youkia.pvz.youkia.com/pvz/php_xml/tool.xml?1660639233132
    fn get_tools(&self) -> Vec<Tool>;

    // http://s36.youkia.pvz.youkia.com/pvz/php_xml/organism.xml?1660639233148
    fn get_oganisms(&self) -> Vec<Organism>;
}
