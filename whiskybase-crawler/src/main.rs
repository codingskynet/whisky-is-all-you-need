use anyhow::Result;
use futures::StreamExt;
use std::time::Duration;
use voyager::{Collector, CrawlerConfig, RequestDelay};

use whiskybase::{WhiskybaseScraper, WhiskybaseState};

mod util;
mod whiskybase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CrawlerConfig::default().allow_domain_with_delay(
        "www.whiskybase.com",
        RequestDelay::Fixed(Duration::from_millis(10)),
    );
    let mut collector = Collector::new(WhiskybaseScraper::default(), config);

    collector.crawler_mut().visit_with_state(
        "https://www.whiskybase.com/sitemaps/sitemaps.xml",
        WhiskybaseState::RootSitemap,
    );

    while let Some(output) = collector.next().await {
        if let Ok(whisky) = output {
            println!("{:?}", whisky);
        }
    }

    Ok(())
}
