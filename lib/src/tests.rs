
use std::{path::{PathBuf}, env::current_dir};

use crate::{game::sys::Quality};
use util::*;

mod util;

fn get_current_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        current_dir().unwrap() //.join("tools")
    } else {
        current_dir().unwrap()
    }
}

#[tokio::test]
async fn test_quality_up() -> Result<(), Box<dyn std::error::Error>> {

    let client = load_nmh().await?;

    let to_quality = Quality::魔神;

    let plant_ids = [ 1996336.,  ];

    for p in plant_ids {
        client.quality_up_to(p, |_, q| q == to_quality).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_skill_up() -> Result<(), Box<dyn std::error::Error>> {

    let client = load_nmh().await?;

    let plant_ids = [1916696., ];
    let skill_ids = [586.,];
    let ups = [10,];

    for (p, (sk, up)) in plant_ids.into_iter()
        .zip(
            skill_ids.into_iter().zip(ups.into_iter())
        ) {
        client.skill_up_to(p, sk, |_, uped| uped == up).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_duty_rewards() -> Result<(), Box<dyn std::error::Error>> {

    let client = load_nmh().await?;

    let duties = [ 21., 22., 25., 26., 32., 31., 42., 41., 40., ]
        .into_iter()
        .chain((2..=20).map(|n| n as f64))
        .chain((70..=77).map(|n| n as f64))
        .chain((63..=69).map(|n| n as f64));
    // let duties = [72., 73., 74., 16., 17., 18., 19., 3., 4., 5., 6., 7., 29., 30., 31., 36., 37., 38., 39., 40., 41., ]
    //     .into_iter();

    client.get_duty_rewards(duties, 3.).await?;

    Ok(())
}

#[tokio::test]
async fn test_fuben_rewards() -> Result<(), Box<dyn std::error::Error>> {

    let client = load_nmh().await?;

    // // client.reset_fuben_reward(1.).await?;
    // // client.reset_fuben_reward(3.).await?;
    // // client.reset_fuben_reward(4.).await?;
    // // client.reset_fuben_reward(5.).await?;
    // // client.reset_and_get_fuben_reward(2., 300).await?;
    client.reset_and_get_fuben_reward(4., 1000).await?;

    Ok(())
}

#[tokio::test]
async fn test_fuben_challenge() -> Result<(), Box<dyn std::error::Error>> {

    let client = load_nmh().await?;

    let fuben_id = 58.;
    let plant_ids = vec![1901265.];

    client.challenge_fuben_repeat(fuben_id, plant_ids, 16).await?;

    Ok(())
}

#[tokio::test]
async fn test_current_dir() {
    println!("{:?}", get_current_dir());
    // println!("{:?}", current_dir().unwrap().as_os_str())
}
