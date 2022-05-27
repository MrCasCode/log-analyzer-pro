use regex::{Captures, Regex};

use crate::models::log_line::LogLine;

/// Creates a default log line assigning the line content to payload and the index
fn default_log_line(line: &str, index: usize) -> LogLine {
    LogLine {
        index: index.to_string(),
        payload: line.to_string(),
        color: None,
        ..Default::default()
    }
}

/// Apply the given format (if any) to the given line
pub fn apply_format(format: &Option<&Regex>, line: &str, index: usize) -> LogLine {
    match format {
        Some(format) => match format.captures(line) {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_content_to_payload_if_no_format() {
        let line = "Test";
        let log_line = apply_format(&None, line, 0);
        assert_eq!(line, log_line.payload)
    }

    #[test]
    fn assign_content_to_payload_if_no_matches() {
        let line = "Test";
        let log_line = apply_format(&Some(&Regex::new("\\d").unwrap()), line, 0);
        assert_eq!(line, log_line.payload)
    }

    #[test]
    fn test_format() {
        let line = "2022-05-27 [1234] test INFO assign_content_to_payload_if_no_matches testing if formatting works";
        let re = Regex::new("(?P<DATE>[\\d]{4}-[\\d]{2}-[\\d]{2}) \\[(?P<TIMESTAMP>[\\d]{4})\\] (?P<APP>[\\w]*) (?P<SEVERITY>[\\w]*) (?P<FUNCTION>[\\w_]*) (?P<PAYLOAD>.*)").unwrap();
        let log_line = apply_format(&Some(&re), line, 0);
        assert_eq!("2022-05-27", log_line.date);
        assert_eq!("1234", log_line.timestamp);
        assert_eq!("test", log_line.app);
        assert_eq!("INFO", log_line.severity);
        assert_eq!("assign_content_to_payload_if_no_matches", log_line.function);
        assert_eq!("testing if formatting works", log_line.payload);
    }
}