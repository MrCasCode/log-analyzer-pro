use crate::models::{
    filter::{Filter, FilterAction},
    format::Format,
    log_line::LogLine,
};
use std::sync::RwLock;

use rustc_hash::FxHashMap as HashMap;

pub trait ProcessingStore {
    fn add_format(&self, id: String, format: String);
    fn get_format(&self, id: &String) -> Option<String>;
    fn get_formats(&self) -> Vec<Format>;
    fn add_filter(&self, id: String, filter: LogLine, action: FilterAction, enabled: bool);
    fn get_filters(&self) -> Vec<(bool, Filter)>;
    fn toggle_filter(&self, id: &String);
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
            formats: RwLock::new(HashMap::default()),
            filters: RwLock::new(HashMap::default()),
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

    fn get_filters(&self) -> Vec<(bool, Filter)> {
        let r = self.filters.read().unwrap();

        let filters = r
            .iter()
            .map(|(id, (action, filter, enabled))| (*enabled, Filter {
                alias: id.clone(),
                action: action.clone(),
                filter: filter.clone(),
            }))
            .collect();

        filters
    }

    fn toggle_filter(&self, id: &String) {
        let mut w = self.filters.write().unwrap();
        if let Some((_, _, enabled)) = w.get_mut(id) {
            *enabled = !*enabled
        }
    }
}
