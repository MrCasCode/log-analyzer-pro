use crate::models::{
    filter::{FilterAction, LogFilter},
    log_line::LogLine,
};

/// Applies the given filter to a line deciding if the filtering requirements are satisfied
/// and applying the filter color if needed
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

/// Apply a list of filters to a line
///
/// Filters are clasified in INCLUDE, EXCLUDE or just MARK
/// and the filtering process follows that priority.
///
/// * If a line is to be included -> It is included
/// * If a line is to be excluded (and it's not previously included) -> It is excluded
/// * Marker filters are applied after to determine the final color
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
    use crate::models::filter::Filter;

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
                date: "2022-01-".to_string(),
                color: Some((255, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                timestamp: "200".to_string(),
                color: Some((254, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                app: "python".to_string(),
                color: Some((253, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                severity: "INFO".to_string(),
                color: Some((252, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                function: "call".to_string(),
                color: Some((251, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });
        run_test(filter, line.clone());

        filter = LogFilter::from(Filter {
            filter: LogLine {
                index: "0".to_string(),
                payload: "some use".to_string(),
                color: Some((250, 0, 0)),
                ..Default::default()
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
                date: "2022-01-".to_string(),
                timestamp: "100".to_string(),
                color: Some((255, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        });

        let is_match = filter_line(&filter, &mut line);
        assert_eq!(is_match, false);
        assert_ne!(filter.color, line.color);
    }
}
