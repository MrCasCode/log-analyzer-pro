use std::{collections::HashMap, sync::Arc, iter::Iterator};

use std::sync::RwLock;

use crate::services::log_source::LogSource;
use async_trait::async_trait;
use futures::join;

pub trait LogStore {
    fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, format: &String, enabled: bool);
    fn add_line(&self, log_id: &String, line: &String);
    fn get_format(&self, log_id: &String) -> Option<String>;
    fn get_logs(&self) -> Vec<(bool, String, String)>;
    fn get_lines(&self, log_id: &String) -> Vec<String>;
    fn extract_lines(&self, log_id: &String) -> Vec<String>;
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

impl LogStore for InMemmoryLogStore {
    fn add_log(&self, log_id: &String, log_source: Arc<Box<dyn LogSource + Send + Sync>>, format: &String, enabled: bool) {
        let (mut source_lock, mut format_lock, mut enabled_lock) =
            (self.source.write().unwrap(),
            self.format.write().unwrap(),
            self.enabled.write().unwrap());

        source_lock.insert(log_id.clone(), log_source);
        format_lock.insert(log_id.clone(), format.clone());
        enabled_lock.insert(log_id.clone(), enabled);
    }

    fn add_line(&self, log_id: &String, line: &String) {
        let mut raw_lines_lock = self.raw_lines.write().unwrap();

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.clone(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        raw_lines.push(line.clone());
    }

    fn get_lines(&self, log_id: &String) -> Vec<String> {
        match self.raw_lines.read().unwrap().get(log_id) {
            Some(lines) => lines.clone(),
            _ => Vec::new()
        }
    }

    fn extract_lines(&self, log_id: &String) -> Vec<String> {
        let mut w = self.raw_lines.write().unwrap();
        let lines = std::mem::take(w.get_mut(log_id).unwrap());

        lines
    }

    fn get_logs(&self) -> Vec<(bool, String, String)> {
        let (format_lock, enabled_lock) = (
            self.format.read().unwrap(),
            self.enabled.read().unwrap(),
        );

        let logs: Vec<(bool, String, String)> = std::iter::zip(format_lock.iter(), enabled_lock.values().into_iter()).map(|((path, format_alias), enabled)| (enabled.clone(), path.clone(), format_alias.clone())).collect();
        logs
    }

    fn get_format(&self, log_id: &String) -> Option<String> {
        let format_lock = self.format.read().unwrap();
        match format_lock.get(log_id) {
            Some(alias) => Some(alias.clone()),
            _ => None
        }
    }
}