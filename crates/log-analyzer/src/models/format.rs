use anyhow::{Result, anyhow};
use regex::Regex;

pub struct Format {
    pub alias: String,
    pub regex: String,
    re: Regex
}

impl Format {
    pub fn new(alias: &String, regex: &String) -> Result<Self> {
        if alias.is_empty() || regex.is_empty() {
            return Err(anyhow!("Error when creating new format.\nPlease review alias and regex are not empty"));
        }

        let re = Regex::new(&regex);
        match re {
            Ok(r) => Ok(Format{alias: alias.clone(), regex :  regex.clone(), re: r}),
            Err(_) => Err(anyhow!("Could not compile regex.\nPlease review regex syntax"))
        }
    }
}