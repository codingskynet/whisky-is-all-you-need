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
pub enum WhiskyAuctioneerState {
    RootSitemap,
    SubSitemap,
    WhiskyPage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhiskyAuctioneerWhisky {
    pub name: String,
    pub price: Price,
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

pub struct WhiskyAuctioneerScraper {
    scraper: DefaultScraper,
    whisky_name: Selector,
    price: Selector,
    reservation: Selector,
    whisky_detail: Selector,
}

impl Default for WhiskyAuctioneerScraper {
    fn default() -> Self {
        Self {
            scraper: DefaultScraper::default(),
            whisky_name: Selector::parse("#new-layout > div.box-outer > div.right > div.left-heading > h1").unwrap(),
            price: Selector::parse("#new-layout > div.box-outer > div.right > div.place-bid.bid-section.bid-info > div.amount.winning > span").unwrap(),
            reservation: Selector::parse("#new-layout > div.box-outer > div.right > div.place-bid.bid-section.bid-info > div.reserve-price").unwrap(),
            whisky_detail: Selector::parse("#new-layout > div.productbuttom > div.left > div.topvbn > div").unwrap()
        }
    }
}

impl Scraper for WhiskyAuctioneerScraper {
    type Output = WhiskyAuctioneerWhisky;
    type State = WhiskyAuctioneerState;

    fn scrape(
        &mut self,
        response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        let html = response.html();

        if let Some(state) = response.state {
            match state {
                WhiskyAuctioneerState::RootSitemap => {
                    let sites = html
                        .select(&self.scraper.sitemap)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskyAuctioneerState::SubSitemap);
                    }
                }
                WhiskyAuctioneerState::SubSitemap => {
                    let sites = html
                        .select(&self.scraper.url)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        if !site.contains("lot/") {
                            continue;
                        }
                        crawler.visit_with_state(&site, WhiskyAuctioneerState::WhiskyPage);
                    }
                }
                WhiskyAuctioneerState::WhiskyPage => {
                    let detail = html
                        .select(&self.whisky_detail)
                        .next()
                        .unwrap()
                        .text()
                        .map(|s| s.trim())
                        .collect::<Vec<_>>();

                    return Ok(Some(WhiskyAuctioneerWhisky {
                        name: select_one_text_from_html(&html, &self.whisky_name).unwrap(),
                        price: Price::from_string(
                            select_one_text_from_html(&html, &self.price).unwrap(),
                        )
                        .unwrap(),
                        reservation: select_one_text_from_html(&html, &self.reservation)
                            .map(|s| s.contains("RESERVE HAS BEEN MET")),
                        distilery: select_one_text_by_column(&detail, "Distillery:")
                            .filter(|s| s != "N/A"),
                        age: select_one_text_by_column(&detail, "Age:")
                            .filter(|s| s != "N/A")
                            .and_then(|s| split_nums_and_strs(s).0.first().cloned())
                            .and_then(|s| s.parse().ok()),
                        vintage: select_one_text_by_column(&detail, "Vintage:")
                            .filter(|s| s != "N/A")
                            .map(|s| WeakDate::from_year(s)),
                        region: select_one_text_by_column(&detail, "Region:")
                            .filter(|s| s != "N/A"),
                        bottler: select_one_text_by_column(&detail, "Bottler:")
                            .filter(|s| s != "N/A"),
                        cask_type: select_one_text_by_column(&detail, "Cask Type:")
                            .filter(|s| s != "N/A"),
                        abv: select_one_text_by_column(&detail, "Bottled Strength:")
                            .filter(|s| s != "N/A")
                            .filter(|s| s.contains("%"))
                            .and_then(|s| split_nums_and_strs(s).0.clone().first().cloned())
                            .and_then(|s| s.parse().ok()),
                        bottle_size: select_one_text_by_column(&detail, "Bottle Size::")
                            .filter(|s| s != "N/A"),
                    }));
                }
            }
        }

        Ok(None)
    }
}

pub fn scrape_whiskyauctioneer(random_delay: Duration) -> Collector<WhiskyAuctioneerScraper> {
    let config = CrawlerConfig::default()
        .allow_domain_with_delay("whiskyauctioneer.com", RequestDelay::random(random_delay));
    let mut collector = Collector::new(WhiskyAuctioneerScraper::default(), config);

    collector.crawler_mut().visit_with_state(
        "https://whiskyauctioneer.com/sitemap.xml",
        WhiskyAuctioneerState::RootSitemap,
    );

    collector
}
