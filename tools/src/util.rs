use lib::{Client, AccountInfo};

#[allow(dead_code)]
pub async fn load_errw() -> lib::Result<Client> {
    // use std::env::current_dir;
    // println!("{:?}", current_dir().unwrap().as_os_str());

    let account = AccountInfo::from_file(
        "F:\\下载\\AB全助手 (无需挂级)\\hack\\tools\\data\\ewrr-s36.toml"
    ).await?;
    let client = Client::builder()
        .account(account)
        .build()?;
    Ok(client)
}

#[allow(dead_code)]
pub async fn load_nmh() -> lib::Result<Client> {
    let account = AccountInfo::from_file(
        "F:\\下载\\AB全助手 (无需挂级)\\hack\\tools\\data\\nmh-s6.toml"
    ).await?;
    let client = Client::builder()
        .account(account)
        .build()?;
    Ok(client)
}