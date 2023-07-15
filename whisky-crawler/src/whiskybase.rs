use std::time::Duration;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use voyager::{
    scraper::Selector, Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper,
};

use crate::{
    data::{DefaultScraper, WeakDate},
    util::{
        select_one_text, select_one_text_by_column, select_one_text_from_html, split_nums_and_strs,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhiskyBaseWhisky {
    pub name: String,
    pub distillery: Option<String>,
    pub bottler: Option<String>,
    pub abv: Option<f32>,
    pub vintage: Option<WeakDate>,
    pub bottled: Option<WeakDate>,
    pub whiskybase_id: String,
    pub whiskybase_score: String,
}

#[derive(Debug)]
pub enum WhiskybaseState {
    RootSitemap,
    SubSitemap,
    WhiskyPage,
}

pub struct WhiskybaseScraper {
    scraper: DefaultScraper,
    whisky_title: Selector,
    whisky_detail: Selector,
    whiskybase_score: Selector,
}

impl Default for WhiskybaseScraper {
    fn default() -> Self {
        Self {
            scraper: DefaultScraper::default(),
            whisky_title: Selector::parse("h1").unwrap(),
            whisky_detail: Selector::parse("div#whisky-details dl").unwrap(),
            whiskybase_score: Selector::parse("span.votes-rating-current").unwrap(),
        }
    }
}

// Whiskybase uses data format 'DD.MM.YYYY' or 'MM.YYYY' or 'YYYY'
fn parse_date(value: String) -> Option<WeakDate> {
    let values = value.split('.').into_iter().collect::<Vec<_>>();

    match values[..] {
        [day, month, year] => {
            if let Some(year) = year.parse().ok() {
                Some(WeakDate {
                    year,
                    month: month.parse().ok(),
                    day: day.parse().ok(),
                })
            } else {
                None
            }
        }
        [month, year] => {
            if let Some(year) = year.parse().ok() {
                Some(WeakDate {
                    year,
                    month: month.parse().ok(),
                    day: None,
                })
            } else {
                None
            }
        }
        [year] => {
            if let Some(year) = year.parse().ok() {
                Some(WeakDate {
                    year,
                    month: None,
                    day: None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_abv(value: String) -> Option<f32> {
    let (nums, _) = split_nums_and_strs(value);

    nums.first().and_then(|n| n.parse().ok())
}

impl Scraper for WhiskybaseScraper {
    type Output = WhiskyBaseWhisky;
    type State = WhiskybaseState;

    fn scrape(
        &mut self,
        response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        let html = response.html();

        if let Some(state) = response.state {
            match state {
                WhiskybaseState::RootSitemap => {
                    let sites = html.select(&self.scraper.sitemap).filter_map(|el| {
                        let site = select_one_text(&el, &self.scraper.loc).unwrap();

                        // the root sitemap has not only whiskies info sitemap, but also other info sitemap.
                        if site.contains("whiskies") {
                            Some(site)
                        } else {
                            None
                        }
                    });

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskybaseState::SubSitemap);
                    }
                }
                WhiskybaseState::SubSitemap => {
                    let sites = html
                        .select(&self.scraper.url)
                        .map(|el| select_one_text(&el, &self.scraper.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskybaseState::WhiskyPage);
                    }
                }
                WhiskybaseState::WhiskyPage => {
                    let whitespace = Regex::new(r"[\t\r\n\s]+").unwrap();

                    let raw_name = html
                        .select(&self.whisky_title)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>();

                    let detail = html
                        .select(&self.whisky_detail)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<Vec<_>>();

                    return Ok(Some(WhiskyBaseWhisky {
                        name: whitespace.replace_all(&raw_name, " ").trim().to_string(),
                        distillery: select_one_text_by_column(&detail, "Distillery"),
                        bottler: select_one_text_by_column(&detail, "Bottler"),
                        abv: select_one_text_by_column(&detail, "Strength")
                            .and_then(|abv| parse_abv(abv)),
                        vintage: select_one_text_by_column(&detail, "Vintage")
                            .and_then(|date| parse_date(date)),
                        bottled: select_one_text_by_column(&detail, "Bottled")
                            .and_then(|date| parse_date(date)),
                        whiskybase_id: select_one_text_by_column(&detail, "Whiskybase ID").unwrap(),
                        whiskybase_score: select_one_text_from_html(&html, &self.whiskybase_score)
                            .unwrap(),
                    }));
                }
            }
        }

        Ok(None)
    }
}

pub fn scrape_whiskybase(random_delay: Duration) -> Collector<WhiskybaseScraper> {
    let config = CrawlerConfig::default()
        .allow_domain_with_delay("www.whiskybase.com", RequestDelay::random(random_delay));
    let mut collector = Collector::new(WhiskybaseScraper::default(), config);

    collector.crawler_mut().visit_with_state(
        "https://www.whiskybase.com/sitemaps/sitemaps.xml",
        WhiskybaseState::RootSitemap,
    );

    collector
}

#[cfg(test)]
mod tests {
    use crate::{
        data::WeakDate,
        whiskybase::{parse_abv, parse_date},
    };

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("06.04.1987".to_string()),
            Some(WeakDate {
                year: 1987,
                month: Some(4),
                day: Some(6)
            })
        );

        assert_eq!(
            parse_date("09.2011".to_string()),
            Some(WeakDate {
                year: 2011,
                month: Some(9),
                day: None
            })
        );

        assert_eq!(
            parse_date("1946".to_string()),
            Some(WeakDate {
                year: 1946,
                month: None,
                day: None
            })
        );
    }

    #[test]
    fn test_parse_abv() {
        assert_eq!(parse_abv("52.0 % Vol.".to_string()), Some(52.0))
    }
}
