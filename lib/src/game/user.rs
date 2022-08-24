
use crate::game::*;
use crate::game::sys::{Quality};

use serde::Deserialize;

use super::sys::{Organism, get_sys_organisms, Tool, get_sys_tools};

#[derive(Debug, Deserialize)]
pub struct UserOrganism {
    pub id: Id,

    /// 对应的`sys::Organism`的Id
    pub target_id: Id,

    pub quality: Quality,
    pub skills: Vec<Skill>,
    pub special_skill: Option<Skill>,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub id: Id,
    pub name: String,
    pub grade: u32,
}

impl UserOrganism {
    pub async fn get_organism(&self) -> Result<&Organism> {
        let o = get_sys_organisms()?;
        for organism in o[..self.target_id].iter().rev() {
            if organism.id == self.target_id {
                unsafe {
                    // safe operation, because sys_info's lifetime is almost equal to the whole app.
                    return Ok(&*(organism as *const Organism));
                }
            }
            if organism.id < self.target_id {
                break;
            }
        }
        Err(ErrorKind::other_str("unknow organism."))
    }
}

#[derive(Debug, Deserialize)]
pub struct UserTool {
    /// 对应的`sys::Tool`的Id
    pub id: Id,
    pub amount: usize,
}

impl UserTool {
    pub async fn get_tool(&self) -> Result<&Tool> {
        let ts = get_sys_tools().await?;
        for tool in ts[..self.id].iter().rev() {
            if tool.tool_id == self.id {
                unsafe {
                    // safe operation, because sys_info's lifetime is almost equal to the whole app.
                    return Ok(&*(tool as *const Tool));
                }
            }
            if tool.tool_id < self.id {
                break;
            }
        }
        Err(ErrorKind::other_str("unknow tool."))
    }
}

pub trait GetUserInfo {
    // /pvz/index.php/Warehouse/index/sig/11c58a61121e4a8b1f77abf6f0f5a1fa?1660726559561
    fn get_warehouse(&self) -> (Vec<UserTool>, Vec<UserOrganism>);
}

