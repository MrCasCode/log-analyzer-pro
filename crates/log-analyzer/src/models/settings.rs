use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{filter::Filter, format::Format};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub formats: Option<Vec<Format>>,
    pub filters: Option<Vec<Filter>>,
    pub primary_color: Option<(u8, u8, u8)>,
}

impl Settings {
    pub fn from_json(json: &str) -> Result<Self> {
        let settings: Result<Settings, _> = serde_json::from_str(json);

        match settings {
            Ok(settings) => Ok(settings),
            _ => Err(anyhow!("Unable to decode settings from file")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::log_line::LogLine;

    use super::*;

    #[test]
    fn test_load_settings() {
        let json = r#"{
            "primary_color": [200, 200, 0],
            "formats": [
                {
                    "alias": "Default",
                    "regex": "(?P<PAYLOAD>.*)"
                }
            ],
            "filters": [
                {
                    "alias": "Name",
                    "action": "INCLUDE",
                    "filter": {
                        "payload": ".*"
                    }
                }
            ]
        }"#;

        let settings: Result<Settings, serde_json::Error> = serde_json::from_str(json);
        assert!(settings.is_ok())
    }

    #[test]
    fn test_load_empty_settings() {
        let json = r#"{}"#;

        let settings: Result<Settings, serde_json::Error> = serde_json::from_str(json);
        assert!(settings.is_ok())
    }

    #[test]
    fn test_serialize_settings() {
        let settings = Settings {
            formats: None,
            filters: Some(vec![Filter {
                alias: "test".into(),
                action: crate::models::filter::FilterAction::INCLUDE,
                filter: LogLine {
                    payload: "test".into(),
                    color: Some((200, 200, 0)),
                    ..Default::default()
                },
            }]),
            primary_color: None,
        };
        let json = serde_json::to_string(&settings);
        assert!(json.is_ok());
    }
}
