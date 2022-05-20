use regex::{Captures, Regex};

use crate::models::log_line::LogLine;

fn default_log_line(line: &String, index: usize) -> LogLine {
    LogLine {
        index: index.to_string(),
        date: "".to_string(),
        timestamp: "".to_string(),
        app: "".to_string(),
        severity: "".to_string(),
        function: "".to_string(),
        payload: line.clone(),
        color: None,
    }
}

pub fn apply_format(format: &Option<&Regex>, line: &String, index: usize) -> LogLine {
    match format {
        Some(format) => match format.captures(&line) {
            Some(captures) => {
                let unwrap_or_empty_string = |capture: &Captures, key: &str| -> String {
                    let str = match capture.name(key) {
                        Some(m) => m.as_str(),
                        None => "",
                    };

                    str.to_string()
                };

                LogLine {
                    index: index.to_string(),
                    date: unwrap_or_empty_string(&captures, "DATE"),
                    timestamp: unwrap_or_empty_string(&captures, "TIMESTAMP"),
                    app: unwrap_or_empty_string(&captures, "APP"),
                    severity: unwrap_or_empty_string(&captures, "SEVERITY"),
                    function: unwrap_or_empty_string(&captures, "FUNCTION"),
                    payload: unwrap_or_empty_string(&captures, "PAYLOAD"),
                    color: None,
                }
            }
            _ => default_log_line(line, index),
        },
        _ => default_log_line(line, index),
    }
}
