use serde::{Deserialize, Serialize};
use voyager::scraper::Selector;

use crate::util::split_nums_and_strs;

pub struct DefaultScraper {
    pub sitemap: Selector,
    pub loc: Selector,
    pub lastmod: Selector,
    pub url: Selector,
}

impl Default for DefaultScraper {
    fn default() -> Self {
        Self {
            sitemap: Selector::parse("sitemap").unwrap(),
            loc: Selector::parse("loc").unwrap(),
            lastmod: Selector::parse("lastmod").unwrap(),
            url: Selector::parse("url").unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WeakDate {
    pub year: u16,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl WeakDate {
    pub fn new(year: u16, month: Option<u8>, day: Option<u8>) -> Self {
        Self { year, month, day }
    }

    pub fn from_year(s: String) -> Self {
        Self::new(s.parse().unwrap(), None, None)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Price {
    pub value: f64,
    pub currency: Currency,
}

impl Price {
    pub fn new(value: f64, currency: Currency) -> Self {
        Self { value, currency }
    }

    pub fn from_string(s: String) -> Option<Self> {
        let (nums, strs) = split_nums_and_strs(s);

        let value: f64 = nums.first().and_then(|n| n.replace(",", "").parse().ok())?;

        let currency = strs
            .iter()
            .flat_map(|s| s.chars().collect::<Vec<char>>())
            .filter_map(|s| Currency::from_symbol(&s))
            .next()?;

        Some(Self::new(value, currency))
    }
}

// from https://en.wikipedia.org/wiki/Currency_symbol, https://en.wikipedia.org/wiki/ISO_4217
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Currency {
    GBP,
    KRW,
    USD,
    EUR,
    JPY,
}

impl Currency {
    fn from_symbol(symbol: &char) -> Option<Self> {
        match symbol {
            '£' => Some(Currency::GBP),
            '₩' => Some(Currency::KRW),
            '$' => Some(Currency::USD),
            '€' => Some(Currency::EUR),
            '¥' => Some(Currency::JPY),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::data::{Currency, Price};

    #[test]
    fn test_parse_price() {
        assert_eq!(
            Price::from_string("£21,500".to_string()),
            Some(Price::new(21500f64, Currency::GBP))
        );

        assert_eq!(
            Price::from_string("$98.99".to_string()),
            Some(Price::new(98.99f64, Currency::USD))
        );

        assert_eq!(
            Price::from_string("1,280,000₩".to_string()),
            Some(Price::new(1_280_000f64, Currency::KRW))
        );
    }
}
