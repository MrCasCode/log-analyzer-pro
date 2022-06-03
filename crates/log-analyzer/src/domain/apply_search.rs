use regex::Regex;

use crate::models::log_line::LogLine;

/// Tries to match the given search expression to all fields of the log
pub fn apply_search(search: &Regex, log_line: &LogLine) -> bool {
    log_line.into_iter().rev().any(|str| search.is_match(str))
}

/// Embed group information in the log line fields.
/// This is used to display formated text.
///
/// The string fields are serialized into json in the form of
/// `[(Option<Group>, Content), ...]`. The group can be used to later be matched
/// with a color in the Front End
pub fn format_search(search: &Regex, log_line: &LogLine) -> LogLine {
    let columns: Vec<String> = LogLine::columns()
        .into_iter()
        .map(|column| {
            let s = log_line.get(&column).unwrap();
            let mut groups = vec![];
            if let Some(m) = search.captures(s) {
                // Capture all matched groups
                for name in search.capture_names() {
                    if let Some(group) = name {
                        if let Some(capture) = m.name(group) {
                            groups.push((group, (capture.start(), capture.end())))
                        }
                    }
                }

                let mut string_groups = vec![];

                let mut offset = 0;
                for (group, (start, end)) in groups {
                    let unmatched = &s[offset..start];
                    if !unmatched.is_empty() {
                        string_groups.push((None, unmatched));
                    }
                    string_groups.push((Some(group), &s[start..end]));
                    offset = end;
                }

                if offset < (s.len() - 1) {
                    string_groups.push((None, &s[offset..]));
                }
                return match serde_json::to_string(&string_groups) {
                    Ok(serialized) => serialized,
                    Err(_) => format!(r#"[[null, "{}"]]"#, s),
                };
            }
            return format!(r#"[[null, "{}"]]"#, s);
        })
        .collect();

    LogLine {
        log: columns[0].clone(),
        index: columns[1].clone(),
        date: columns[2].clone(),
        timestamp: columns[3].clone(),
        app: columns[4].clone(),
        severity: columns[5].clone(),
        function: columns[6].clone(),
        payload: columns[7].clone(),
        color: log_line.color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_formatting() {
        let line = LogLine {
            log: "test.log".into(),
            index: "0".into(),
            date: "2022-06-02".into(),
            timestamp: "42".into(),
            app: "test".into(),
            severity: "INFO".into(),
            function: "test_format".into(),
            payload: "Highlighting search matches is going to be awesome, I tell you".into(),
            color: None,
        };

        let regex = Regex::new("(?P<BLACK>awesome)").unwrap();

        let formatted_line = format_search(&regex, &line);
        let unformat: Vec<(Option<&str>, &str)> = serde_json::from_str(&formatted_line.payload).unwrap();

        // We expect 3 groups since they are splitted by the formatted block "awesome"
        assert!(unformat.len() == 3);
        // The second block "awesome" is formatted with the "BLACK" group
        assert!(unformat[1].0 == Some("BLACK"));
        assert!(unformat[1].1 == "awesome");
    }
}
