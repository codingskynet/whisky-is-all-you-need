use anyhow::Result;
use futures::StreamExt;
use std::time::Duration;
use whiskyauction::scrape_whiskyauction;
use whiskyauctioneer::scrape_whiskyauctioneer;
use whiskybase::scrape_whiskybase;

mod data;
mod util;
mod whiskyauction;
mod whiskyauctioneer;
mod whiskybase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut whiskyauctioneer_whiskies = scrape_whiskyauctioneer(Duration::from_secs(10));

    while let Some(result) = whiskyauctioneer_whiskies.next().await {
        println!("{result:?}")
    }

    let mut whiskybase_whiskies = scrape_whiskybase(Duration::from_millis(10));

    while let Some(result) = whiskybase_whiskies.next().await {
        println!("{result:?}")
    }

    let mut whiskyauction_whiskies = scrape_whiskyauction(Duration::from_millis(50));

    while let Some(result) = whiskyauction_whiskies.next().await {
        println!("{result:?}")
    }

    Ok(())
}
