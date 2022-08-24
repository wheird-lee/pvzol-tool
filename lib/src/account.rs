use std::{collections::HashMap, path::Path};

use crate::Result;

use serde::Deserialize;
use tokio::{fs::read};

#[derive(Deserialize)]
pub struct AccountInfo {
    pub server: usize,
    pub cookies: HashMap<String, String>
}

impl AccountInfo {
    pub async fn from_file(file_name: impl AsRef<Path>) -> Result<AccountInfo> {
        let file_content = read(file_name).await?;
        AccountInfo::from_bytes(file_content.as_slice())
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<AccountInfo> {
        Ok(toml::from_slice(bytes.as_ref())?)
    }
}
