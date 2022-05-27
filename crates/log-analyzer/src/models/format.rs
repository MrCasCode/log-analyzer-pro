use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Format {
    pub alias: String,
    pub regex: String
}



impl Format {
    pub fn new(alias: &String, regex: &String) -> Result<Self> {
        if alias.is_empty() || regex.is_empty() {
            return Err(anyhow!("Error when creating new format.\nPlease review alias and regex are not empty"));
        }

        let re = Regex::new(regex);
        match re {
            Ok(_) => Ok(Format{alias: alias.clone(), regex : regex.clone()}),
            Err(_) => Err(anyhow!("Could not compile regex.\nPlease review regex syntax"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let format = Format::new(&"All".to_string(), &"(?P<PAYLOAD>.*)".to_string()).unwrap();
        let json = serde_json::to_string(&format);
        assert!(json.is_ok())
    }

    #[test]
    fn deserialize() {
        let json =r#"{"alias":"All","regex":"(?P<PAYLOAD>.*)"}"#;

        let format: Result<Format, serde_json::Error> = serde_json::from_str(json);
        assert!(format.is_ok())
    }
}