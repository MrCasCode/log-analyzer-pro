use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use super::{format::Format, filter::Filter};


#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub formats: Option<Vec<Format>>,
    pub filters: Option<Vec<Filter>>,
    pub primary_color: Option<HashMap<String, u8>>
}

impl Settings {
    pub fn from_json(json: &str) -> Result<Self> {
        let settings: Result<Settings, _> = serde_json::from_str(json);

        match settings {
            Ok(settings) => Ok(settings),
            _ => Err(anyhow!("Unable to decode settings from file"))
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_settings() {
        let json = r#"{
            "primary_color": {
                "red": 0,
                "green": 200,
                "blue": 200
            },
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
}