use regex::{Regex, Captures};

use crate::models::log_line::LogLine;

pub fn apply_format(format: &String, line: &String) -> Option<LogLine> {
    let re = Regex::new(&format);
    match re {
        Ok(r) => match r.captures(&line) {
            Some(captures) => {
                let unwrap_or_empty_string = |capture: &Captures, key: &str| -> String {
                    let str = match captures.name(key) {
                        Some(m) => m.as_str(),
                        None => "",
                    };

                    str.to_string()
                };

                Some(LogLine {
                    date: unwrap_or_empty_string(&captures, "DATE"),
                    timestamp: unwrap_or_empty_string(&captures, "TIMESTAMP"),
                    app: unwrap_or_empty_string(&captures, "APP"),
                    severity: unwrap_or_empty_string(&captures, "SEVERITY"),
                    function: unwrap_or_empty_string(&captures, "FUNCTION"),
                    payload: unwrap_or_empty_string(&captures, "PAYLOAD"),
                    color: None,
                })
            }
            _ => None,
        },
        Err(_) => None,
    }
}