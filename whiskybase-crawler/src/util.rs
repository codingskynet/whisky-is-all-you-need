use voyager::scraper::{ElementRef, Html, Selector};

pub fn select_one_text_from_html(html: &Html, selector: &Selector) -> Option<String> {
    html.select(selector)
        .next()?
        .text()
        .next()
        .map(|s| s.to_string())
}

pub fn select_one_text(element_ref: &ElementRef, selector: &Selector) -> Option<String> {
    element_ref
        .select(selector)
        .next()?
        .text()
        .next()
        .map(|s| s.to_string())
}

pub fn select_one_text_by_column(vec: &Vec<&str>, column: &str) -> Option<String> {
    let mut iter = vec.iter();

    while let Some(next) = iter.next() {
        if *next == column {
            return Some(iter.next().unwrap().to_string());
        }
    }

    None
}
