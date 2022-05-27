use super::log_line::LogLine;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum FilterAction {
    /// Just add a color marker
    MARKER,
    /// Exclude what is not matched by this filter
    INCLUDE,
    /// Exclude what is matched by this filter
    EXCLUDE,
}

impl From<usize> for FilterAction {
    fn from(v: usize) -> Self {
        match v {
            0 => FilterAction::INCLUDE,
            1 => FilterAction::EXCLUDE,
            _ => FilterAction::MARKER,
        }
    }
}

impl Default for FilterAction {
    fn default() -> Self {
        FilterAction::MARKER
    }
}


#[derive(Default, Clone, Debug)]
pub struct LogFilter {
    pub action: FilterAction,
    pub filters: Vec<(String, Regex)>,
    pub color: Option<(u8, u8, u8)>
}

impl From<Filter> for LogFilter {
    fn from(f: Filter) -> Self {
        Self { action: f.action, filters: f.get_filters(), color: f.filter.color }
    }
}



#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Filter {
    pub alias: String,
    pub action: FilterAction,
    pub filter: LogLine
}

impl Filter {
    /// Get the valid filters from the filter data
    /// Returns a vector of (Key, Regex); Key is to be used with the get method of LogLines
    pub fn get_filters(&self) -> Vec<(String, Regex)> {
        let mut filters = Vec::new();
        for (k, v) in self.filter.values() {
            if let Ok(re) = Regex::new(v) {
                filters.push((k.into(), re))
            }
        }

        filters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let filter = Filter {
            alias: "All".into(),
            action: FilterAction::MARKER,
            filter: LogLine {
                index: "0".to_string(),
                date: "".to_string(),
                timestamp: "".to_string(),
                app: "".to_string(),
                severity: "".to_string(),
                function: "".to_string(),
                payload: "".to_string(),
                color: None,
            },
        };
        let json = serde_json::to_string(&filter);
        assert!(json.is_ok())
    }

    #[test]
    fn deserialize() {
        let json = r#"
        {
            "alias": "Name",
            "action": "INCLUDE",
            "filter": {"payload": ".*"}
        }"#;

        let filter: Result<Filter, serde_json::Error> = serde_json::from_str(json);
        assert!(filter.is_ok())
    }

    #[test]
    fn deserialize_list() {
        let json = r#"[
            {
                "alias": "Name",
                "action": "INCLUDE",
                "filter": {"payload": ".*"}
            },
            {
                "alias": "All",
                "action": "EXCLUDE",
                "filter": {"payload": ".*"}
            }
        ]"#;

        let filter: Result<Vec<Filter>, serde_json::Error> = serde_json::from_str(json);
        assert!(filter.is_ok())
    }
}
