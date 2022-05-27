use regex::Regex;

use crate::models::{
    filter::{Filter, FilterAction, LogFilter},
    log_line::LogLine,
};

fn filter_line<'a>(filtering: &'a LogFilter, log_line: &'a mut LogLine) -> bool {
    let mut is_match = false;
    for (key, re) in &filtering.filters {
        is_match = re.is_match(log_line.get(key).unwrap());
        if is_match == false {
            break;
        }
    }

    if is_match {
        log_line.color = filtering.color;
    }

    is_match
}

pub fn apply_filters(filters: &[LogFilter], mut log_line: LogLine) -> Option<LogLine> {
    let include_filters = filters
        .iter()
        .filter(|filter| filter.action == FilterAction::INCLUDE);
    let exclude_filters = filters
        .iter()
        .filter(|filter| filter.action == FilterAction::EXCLUDE);
    let marker_filters = filters
        .iter()
        .filter(|filter| filter.action == FilterAction::MARKER);

    // If should be included check for any potential override of color with markers and return the line
    for filter in include_filters.clone() {
        if filter_line(&filter, &mut log_line) {
            for marker_filter in marker_filters {
                filter_line(&marker_filter, &mut log_line);
            }

            return Some(log_line);
        }
    }

    // If is not included and is excluded -> exclude it
    for filter in exclude_filters {
        if filter_line(&filter, &mut log_line) {
            return None;
        }
    }

    // If there are no including filters filter it just with markers and return the line
    if include_filters.count() == 0 {
        for filter in marker_filters {
            filter_line(&filter, &mut log_line);
        }

        return Some(log_line);
    }

    // There was including filters but we didnt match. Line not to be included
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_found_on_every_individual_field() {
        let run_test = |filter, mut line| {
            let is_match = filter_line(&filter, &mut line);

            assert_eq!(is_match, true);
            assert_eq!(filter.color, line.color);
        };

        let line = LogLine {
            index: "0".to_string(),
            date: "2022-01-02".to_string(),
            timestamp: "200.05".to_string(),
            app: "python".to_string(),
            severity: "INFO".to_string(),
            function: "call".to_string(),
            payload: "some useful information".to_string(),
            color: None,
        };

        let mut filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "2022-01-".to_string(),
                timestamp: "".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: Some((255, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "200".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: Some((254, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "".to_string(),
                app: "python".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: Some((253, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "".to_string(),
                app: "".to_string(),
                severity: "INFO".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: Some((252, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "call".to_string(),
                payload: "".to_string(),
                color: Some((251, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "some use".to_string(),
                color: Some((250, 0, 0)),
            },
            ..Default::default()
        });
        run_test(filter, line.clone());
    }

    #[test]
    fn dont_match_on_multiple_conditions_unsatisfied() {
        let mut line = LogLine {
            index: "0".to_string(),
            date: "2022-01-02".to_string(),
            timestamp: "200.05".to_string(),
            app: "python".to_string(),
            severity: "INFO".to_string(),
            function: "call".to_string(),
            payload: "some useful information".to_string(),
            color: None,
        };
        let filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                date: "2022-01-".to_string(),
                timestamp: "100".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: Some((255, 0, 0)),
            },
            ..Default::default()
        });

        let is_match = filter_line(&filter, &mut line);
        assert_eq!(is_match, false);
        assert_ne!(filter.color, line.color);
    }
}
