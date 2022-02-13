use std::{collections::HashMap, sync::Arc, iter::Iterator};

use async_std::sync::RwLock;

use crate::services::log_source::LogSource;
use async_trait::async_trait;
use futures::join;

#[async_trait]
pub trait LogStore {
    async fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, format: &String, enabled: bool);
    async fn add_line(&self, log_id: &String, line: &String);
    async fn get_logs(&self) -> Vec<(bool, String, String)>;
}

pub struct InMemmoryLogStore {
    /// K: log_path -> V: lines
    raw_lines: RwLock<HashMap<String, Vec<String>>>,
    /// K: log_path -> V: format
    format: RwLock<HashMap<String, String>>,
    /// K: log_path -> V: enabled
    enabled: RwLock<HashMap<String, bool>>,
    /// K: log_path -> V: source controller
    source: RwLock<HashMap<String, Arc<Box<dyn LogSource + Send + Sync>>>>
}

impl InMemmoryLogStore {
    pub fn new() -> Self {
        Self {
            raw_lines : RwLock::new(HashMap::new()),
            format : RwLock::new(HashMap::new()),
            enabled : RwLock::new(HashMap::new()),
            source : RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl LogStore for InMemmoryLogStore {
    async fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, format: &String, enabled: bool) {
        let (mut source_lock, mut format_lock, mut enabled_lock) = join!(
            self.source.write(),
            self.format.write(),
            self.enabled.write(),
        );

        source_lock.insert(log_id.clone(), log_source);
        format_lock.insert(log_id.clone(), format.clone());
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

    async fn get_logs(&self) -> Vec<(bool, String, String)> {
        let (format_lock, enabled_lock) = join!(
            self.format.read(),
            self.enabled.read(),
        );

        let logs: Vec<(bool, String, String)> = std::iter::zip(format_lock.iter(), enabled_lock.values().into_iter()).map(|((path, format_alias), enabled)| (enabled.clone(), path.clone(), format_alias.clone())).collect();
        logs


    }
}