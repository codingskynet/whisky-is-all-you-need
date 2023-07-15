use regex::Regex;
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

pub fn split_nums_and_strs(s: String) -> (Vec<String>, Vec<String>) {
    let num_re = Regex::new(r"[0-9]+[,?[0-9]*]*\.?[0-9]*").unwrap();
    let str_re = Regex::new(r"[£₩$€¥A-Za-z\ ]+").unwrap();

    (
        num_re
            .find_iter(&s)
            .map(|m| m.as_str().to_string())
            .collect(),
        str_re
            .find_iter(&s)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use crate::util::split_nums_and_strs;

    #[test]
    fn test_split_nums_and_strs() {
        assert_eq!(
            split_nums_and_strs("£21,500".to_string()),
            (vec!["21,500".to_string()], vec!["£".to_string()])
        );

        assert_eq!(
            split_nums_and_strs("30 YEAR OLD".to_string()),
            (vec!["30".to_string()], vec!["YEAR OLD".to_string()])
        );

        assert_eq!(
            split_nums_and_strs("66.3 PROOF".to_string()),
            (vec!["66.3".to_string()], vec!["PROOF".to_string()])
        );

        assert_eq!(
            split_nums_and_strs("A 3.41 BC kkk lkjf $€CC 11,34.1029 OK".to_string()),
            (
                vec!["3.41".to_string(), "11,34.1029".to_string()],
                vec![
                    "A".to_string(),
                    "BC kkk lkjf $€CC".to_string(),
                    "OK".to_string()
                ]
            )
        );
    }
}
