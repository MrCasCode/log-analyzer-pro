use crate::models::{
    filter::{Filter, FilterAction},
    format::Format,
    log_line::LogLine,
};
use std::sync::RwLock;

use std::collections::HashMap;

pub trait ProcessingStore {
    fn add_format(&self, id: String, format: String);
    fn get_format(&self, id: &String) -> Option<String>;
    fn get_formats(&self) -> Vec<Format>;
    fn add_filter(&self, id: String, filter: LogLine, action: FilterAction, enabled: bool);
    fn get_filters(&self) -> Vec<Filter>;
}
pub struct InMemmoryProcessingStore {
    /// Map of <alias, Regex string>
    formats: RwLock<HashMap<String, String>>,
    /// Map of <alias, Filter details>
    filters: RwLock<HashMap<String, (FilterAction, LogLine, bool)>>,
}

impl InMemmoryProcessingStore {
    pub fn new() -> Self {
        Self {
            formats: RwLock::new(HashMap::new()),
            filters: RwLock::new(HashMap::new()),
        }
    }
}

impl ProcessingStore for InMemmoryProcessingStore {
    fn add_format(&self, id: String, format: String) {
        let mut w = self.formats.write().unwrap();
        w.insert(id, format);
    }

    fn get_format(&self, id: &String) -> Option<String> {
        let r = self.formats.read().unwrap();
        match r.get(id) {
            Some(format) => Some(format.clone()),
            _ => None,
        }
    }

    fn get_formats(&self) -> Vec<Format> {
        let formats_lock = self.formats.read().unwrap();
        formats_lock
            .iter()
            .map(|(alias, regex)| Format {
                alias: alias.clone(),
                regex: regex.clone(),
            })
            .collect()
    }

    fn add_filter(&self, id: String, filter: LogLine, action: FilterAction, enabled: bool) {
        let mut w = self.filters.write().unwrap();
        w.insert(id, (action, filter, enabled));
    }

    fn get_filters(&self) -> Vec<Filter> {
        let r = self.filters.read().unwrap();

        let filters = r
            .values()
            .filter(|(_action, _filter, enabled)| *enabled == true)
            .map(|(action, filter, _enabled)| Filter {
                alias: "".to_string(),
                action: action.clone(),
                filter: filter.clone(),
            })
            .collect();

        filters
    }
}
