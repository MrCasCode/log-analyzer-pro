use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use super::{format::Format, filter::Filter};


#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub formats: Vec<Format>,
    pub filters: Vec<Filter>
}

impl Settings {
    pub fn from_json(json: &str) -> Result<Self> {
        let settings: Result<Settings, _> = serde_json::from_str(json);

        match settings {
            Ok(settings) => Ok(settings),
            _ => Err(anyhow!("Unable to decode settings from"))
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_settings() {
        let json = r#"{
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
}