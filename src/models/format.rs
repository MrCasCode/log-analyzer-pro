use regex::Regex;

pub struct Format {
    pub alias: String,
    pub regex: String,
    re: Regex
}

impl Format {
    pub fn new(alias: String, regex: String) -> Option<Self> {
        let re = Regex::new(&regex);
        match re {
            Ok(r) => Some(Format{alias: alias, regex :  regex, re: r}),
            Err(_) => None
        }
    }
}