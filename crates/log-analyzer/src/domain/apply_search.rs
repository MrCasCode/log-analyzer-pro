use regex::Regex;

use crate::models::{log_line::LogLine, log_line_styled::LogLineStyled};

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
pub fn format_search(search: &Regex, log_line: &LogLine) -> LogLineStyled {
    let mut columns: Vec<Vec<(Option<String>, String)>> = LogLine::columns()
        .into_iter()
        .map(|column| {
            let s = log_line.get(&column).unwrap();
            let mut groups = vec![];
            if let Some(m) = search.captures(s) {
                // Capture all matched groups
                for group in search.capture_names().flatten() {
                    if let Some(capture) = m.name(group) {
                        groups.push((group, (capture.start(), capture.end())))
                    }
                }

                let mut string_groups = vec![];

                // If there are captured groups manage the splitting between unformatted and captured parts of the string
                if !groups.is_empty() {
                    let mut offset = 0;
                    for (group, (start, end)) in groups {
                        let unmatched = &s[offset..start];
                        if !unmatched.is_empty() {
                            string_groups.push((None, unmatched.to_string()));
                        }
                        string_groups.push((Some(group.to_string()), s[start..end].to_string()));
                        offset = end;
                    }

                    if offset < (s.len().saturating_sub(1)) {
                        string_groups.push((None, s[offset..].to_string()));
                    }
                }
                // Otherwise just add the entire string without any format
                else {
                    string_groups.push((None, s.to_string()));
                }
                return string_groups;
            }
            return vec![(None, s.to_string())]
        })
        .collect();

    LogLineStyled {
        log: std::mem::take(&mut columns[0]),
        index: std::mem::take(&mut columns[1]),
        date: std::mem::take(&mut columns[2]),
        timestamp: std::mem::take(&mut columns[3]),
        app: std::mem::take(&mut columns[4]),
        severity: std::mem::take(&mut columns[5]),
        function: std::mem::take(&mut columns[6]),
        payload: std::mem::take(&mut columns[7]),
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
            payload: "Highlighting search matches is going to be awesome, I tell you\\".into(),
            ..Default::default()
        };

        let regex = Regex::new("(?P<BLACK>awesome)").unwrap();

        let formatted_line = format_search(&regex, &line);

        // Just to test its not crashing
        let _unformat = formatted_line.unformat();

        // We expect 3 groups since they are splitted by the formatted block "awesome"
        assert!(formatted_line.payload.len() == 3);
        // The second block "awesome" is formatted with the "BLACK" group
        assert!(formatted_line.payload[1].0 == Some("BLACK".to_string()));
        assert!(formatted_line.payload[1].1 == "awesome");
    }
}
