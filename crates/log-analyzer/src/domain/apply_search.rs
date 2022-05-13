use regex::Regex;

use crate::models::log_line::LogLine;

pub fn apply_search(search: &String, log_line: &LogLine) -> bool {
    let re = Regex::new(&search);
    match re {
        Ok(r) => log_line.into_iter().rev().any(|str| r.is_match(str)),
        _ => false,
    }
}
