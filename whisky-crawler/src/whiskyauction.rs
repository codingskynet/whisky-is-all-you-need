use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use voyager::{
    scraper::Selector, Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper,
};

use crate::{
    data::{DefaultScraper, Price, WeakDate},
    util::{
        select_one_text, select_one_text_by_column, select_one_text_from_html, split_nums_and_strs,
    },
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
    pub price: Option<Price>,
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
    whisky_detail: Selector,
    bottled_size_and_abv: Selector,
}

impl Default for WhiskyAuctionScraper {
    fn default() -> Self {
        Self {
            scraper: DefaultScraper::default(),
            whisky_name: Selector::parse("div.product-data > h1 > span.lotName1.line-1").unwrap(),
            price: Selector::parse(
                "div.product-data > div.lot-detail-data > div.hammerprice > div > span.winningBid",
            )
            .unwrap(),
            whisky_detail: Selector::parse(
                "#contentsecondary > div > div.content > div.meta > div.metawrap",
            )
            .unwrap(),
            bottled_size_and_abv: Selector::parse("div.product-data > h1 > span.lotName3.line-3")
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
                        if !site.contains("auctionId") {
                            continue;
                        }

                        crawler.visit_with_state(&site, WhiskyAuctionState::SubSitemap);
                    }
                }
                WhiskyAuctionState::SubSitemap => {
                    let sites = html
                        .select(&self.scraper.url)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskyAuctionState::WhiskyPage);
                    }
                }
                WhiskyAuctionState::WhiskyPage => {
                    let name = select_one_text_from_html(&html, &self.whisky_name).unwrap();

                    let price =
                        select_one_text_from_html(&html, &self.price).filter(|s| s != "N/A");

                    let detail = html
                        .select(&self.whisky_detail)
                        .next()
                        .unwrap()
                        .text()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>();

                    let bottled_size_and_abv =
                        select_one_text_from_html(&html, &self.bottled_size_and_abv)
                            .unwrap()
                            .split("/")
                            .map(|s| s.trim().to_string())
                            .collect::<Vec<String>>();

                    return Ok(Some(WhiskyAuctionWhisky {
                        name,
                        price: price.and_then(Price::from_string),
                        age: select_one_text_by_column(&detail, "Age").and_then(|s| s.parse().ok()),
                        vintage: select_one_text_by_column(&detail, "Vintage")
                            .map(|s| WeakDate::from_year(s)),
                        region: select_one_text_by_column(&detail, "Region"),
                        bottler: select_one_text_by_column(&detail, "Bottler"),
                        cask_type: select_one_text_by_column(&detail, "Cask Type"),
                        abv: bottled_size_and_abv
                            .get(1)
                            .cloned()
                            .and_then(|s| split_nums_and_strs(s).0.first().cloned())
                            .and_then(|s| s.parse().ok()),
                        bottle_size: bottled_size_and_abv.get(0).cloned(),
                    }));
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
