use super::log_line::LogLine;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum FilterAction {
    /// Just add a color marker
    MARKER,
    /// Exclude what is not matched by this filter
    INCLUDE,
    /// Exclude what is matched by this filter
    EXCLUDE
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Filter {
    pub alias: String,
    pub action: FilterAction,
    pub filter: LogLine
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let filter = Filter{alias: "All".into(), action: FilterAction::MARKER, filter: LogLine { date: "".to_string(), timestamp: "".to_string(), app: "".to_string(), severity: "".to_string(), function: "".to_string(), payload: "".to_string(), color: None }};
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