use std::{collections::HashMap, sync::Arc};

use async_std::{sync::RwLock, future};
use serde::de::value::StrDeserializer;

use crate::services::log_source::LogSource;
use async_trait::async_trait;
use futures::join;

#[async_trait]
pub trait LogStore {
    async fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, enabled: bool);
    async fn add_line(&self, log_id: &String, line: &String);
}

pub struct InMemmoryLogStore {
    raw_lines: RwLock<HashMap<String, Vec<String>>>,
    formats: RwLock<HashMap<String, Vec<String>>>,
    enabled: RwLock<HashMap<String, bool>>,
    source: RwLock<HashMap<String, Arc<Box<dyn LogSource + Send + Sync>>>>
}

impl InMemmoryLogStore {
    pub fn new() -> Self {
        Self {
            raw_lines : RwLock::new(HashMap::new()),
            formats : RwLock::new(HashMap::new()),
            enabled : RwLock::new(HashMap::new()),
            source : RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl LogStore for InMemmoryLogStore {
    async fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, enabled: bool) {
        let (mut source_lock, mut enabled_lock) = join!(
            self.source.write(),
            self.enabled.write(),
        );

        source_lock.insert(log_id.clone(), log_source);
        enabled_lock.insert(log_id.clone(), enabled);
    }

    async fn add_line(&self, log_id: &String, line: &String) {
        let mut raw_lines_lock = self.raw_lines.write().await;

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.clone(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        raw_lines.push(line.clone());
    }
}