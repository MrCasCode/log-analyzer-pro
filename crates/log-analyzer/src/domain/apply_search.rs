use regex::Regex;

use crate::models::log_line::LogLine;

pub fn apply_search(search: &Regex, log_line: &LogLine) -> bool {
    log_line.into_iter().rev().any(|str| search.is_match(str))
}
