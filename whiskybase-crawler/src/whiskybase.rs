use anyhow::Result;
use regex::Regex;
use voyager::{scraper::Selector, Crawler, Response, Scraper};

use crate::util::{select_one_text, select_one_text_by_column, select_one_text_from_html};

#[derive(Debug)]
pub struct Whisky {
    name: String,
    distillery: Option<String>,
    whiskybase_id: String,
    whiskybase_score: String,
}

#[derive(Debug)]
pub enum WhiskybaseState {
    RootSitemap,
    WhiskySitemap,
    WhiskyPage,
}

pub struct WhiskybaseScraper {
    sitemap: Selector,
    loc: Selector,
    lastmod: Selector,
    url: Selector,
    whisky_title: Selector,
    whisky_detail: Selector,
    whiskybase_score: Selector,
}

impl Default for WhiskybaseScraper {
    fn default() -> Self {
        Self {
            sitemap: Selector::parse("sitemap").unwrap(),
            loc: Selector::parse("loc").unwrap(),
            lastmod: Selector::parse("lastmod").unwrap(),
            url: Selector::parse("url").unwrap(),
            whisky_title: Selector::parse("h1").unwrap(),
            whisky_detail: Selector::parse("div#whisky-details dl").unwrap(),
            whiskybase_score: Selector::parse("span.votes-rating-current").unwrap(),
        }
    }
}

impl Scraper for WhiskybaseScraper {
    type Output = Whisky;
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
                    let sites = html.select(&self.sitemap).filter_map(|el| {
                        let site = select_one_text(&el, &self.loc).unwrap();

                        // the root sitemap has not only whiskies info sitemap, but also other info sitemap.
                        if site.contains("whiskies") {
                            Some(site)
                        } else {
                            None
                        }
                    });

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskybaseState::WhiskySitemap);
                    }
                }
                WhiskybaseState::WhiskySitemap => {
                    let sites = html
                        .select(&self.url)
                        .map(|el| select_one_text(&el, &self.loc).unwrap());

                    // TODO: If using DB on someday, check lastmod for update

                    for site in sites {
                        crawler.visit_with_state(&site, WhiskybaseState::WhiskyPage);
                    }
                }
                WhiskybaseState::WhiskyPage => {
                    let whitespace = Regex::new(r"[\t\r\n\s]+").unwrap();

                    let raw_title = html
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

                    return Ok(Some(Whisky {
                        name: whitespace.replace_all(&raw_title, " ").trim().to_string(),
                        distillery: select_one_text_by_column(&detail, "Distillery"),
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
