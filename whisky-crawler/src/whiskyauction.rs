use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use voyager::{
    scraper::Selector, Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper,
};

use crate::{
    data::{DefaultScraper, WeakDate},
    util::select_one_text,
};

#[derive(Debug)]
pub enum WhiskyAuctionState {
    RootSitemap,
    SubSitemap,
    WhiskyPage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhiskyAuctionWhisky {
    pub name: String,
    pub price: u32,
    pub reservation: Option<bool>,
    pub distilery: Option<String>,
    pub age: Option<u32>,
    pub vintage: Option<WeakDate>,
    pub region: Option<String>,
    pub bottler: Option<String>,
    pub cask_type: Option<String>,
    pub abv: Option<f32>,
    pub bottle_size: Option<String>,
}

pub struct WhiskyAuctionScraper {
    scraper: DefaultScraper,
    whisky_name: Selector,
    price: Selector,
    reservation: Selector,
    whisky_detail: Selector,
}

impl Default for WhiskyAuctionScraper {
    fn default() -> Self {
        Self {
            scraper: DefaultScraper::default(),
            whisky_name: Selector::parse(
                "#lot_118923 > div.product-data > h1 > span.lotName1.line-1",
            )
            .unwrap(),
            price: Selector::parse(
                "div.product-data > div.lot-detail-data > div.hammerprice > div > span.winningBid",
            )
            .unwrap(),
            reservation: Selector::parse(
                "#lot_40000 > div.product-data > div.lot-statuses > div.lot-status > div",
            )
            .unwrap(),
            whisky_detail: Selector::parse(
                "#contentsecondary > div > div.content > div.meta > div.metawrap",
            )
            .unwrap(),
        }
    }
}

impl Scraper for WhiskyAuctionScraper {
    type Output = WhiskyAuctionWhisky;
    type State = WhiskyAuctionState;

    fn scrape(
        &mut self,
        response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        let html = response.html();

        if let Some(state) = response.state {
            match state {
                WhiskyAuctionState::RootSitemap => {
                    let sites = html
                        .select(&self.scraper.sitemap)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskyAuctionState::SubSitemap);
                    }
                }
                WhiskyAuctionState::SubSitemap => {
                    let sites = html
                        .select(&self.scraper.url)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        println!("{site}");
                        crawler.visit_with_state(&site, WhiskyAuctionState::WhiskyPage);
                    }
                }
                WhiskyAuctionState::WhiskyPage => {
                    let name = html
                        .select(&self.whisky_name)
                        .next()
                        .unwrap()
                        .text()
                        .next()
                        .unwrap();

                    todo!()

                    // return WhiskyAuctioneerWhisky {
                    //     name:
                    //     price:
                    //     reservation:
                    //     distilery:
                    //     age:
                    //     vintage:
                    //     region:
                    //     bottler:
                    //     cask_type:
                    //     abv:
                    //     bottle_size:
                    // };
                }
            }
        }

        Ok(None)
    }
}

pub fn scrape_whiskyauction(random_delay: Duration) -> Collector<WhiskyAuctionScraper> {
    let config = CrawlerConfig::default()
        .allow_domain_with_delay("whisky.auction", RequestDelay::random(random_delay));
    let mut collector = Collector::new(WhiskyAuctionScraper::default(), config);

    collector.crawler_mut().visit_with_state(
        "https://whisky.auction/sitemap.xml",
        WhiskyAuctionState::RootSitemap,
    );

    collector
}
