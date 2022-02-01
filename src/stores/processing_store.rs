use crate::models::{filter::{FilterAction, Filter}, log_line::LogLine};
use async_std::sync::RwLock;
use async_trait::async_trait;
use std::{collections::HashMap, iter::FromIterator, sync::Arc};

#[async_trait]
pub trait ProcessingStore {
    async fn add_format(&self, id: String, format: String);
    async fn get_format(&self, id: &String) -> Option<String>;
    async fn add_filter(&self, id: String, filter: LogLine, action: FilterAction, enabled: bool);
    async fn get_filters(&self) -> Vec<Filter>;
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

#[async_trait]
impl ProcessingStore for InMemmoryProcessingStore {
    async fn add_format(&self, id: String, format: String) {
        let mut w = self.formats.write().await;
        w.insert(id, format);
    }

    async fn get_format(&self, id: &String) -> Option<String> {
        let r = self.formats.read().await;
        match r.get(id) {
            Some(format) => Some(format.clone()),
            _ => None,
        }
    }

    async fn add_filter(&self, id: String, filter: LogLine, action: FilterAction, enabled: bool) {
        let mut w = self.filters.write().await;
        w.insert(id, (action, filter, enabled));
    }

    async fn get_filters(&self) -> Vec<Filter> {
        let r = self.filters.read().await;

        let filters = r
            .values()
            .filter(|(action, filter, enabled)| *enabled == true)
            .map(|(action, filter, enabled)| Filter{ action: action.clone(), filter: filter.clone()})
            .collect();

        filters
    }
}
