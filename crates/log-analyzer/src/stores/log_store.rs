use rustc_hash::FxHashMap as HashMap;
use parking_lot::RwLock;
use std::{iter::Iterator, ops::Range, sync::Arc};

use crate::services::log_source::LogSource;

pub trait LogStore {
    fn add_log(
        &self,
        log_id: &String,
        log_source: Arc<Box<dyn LogSource + Send + Sync>>,
        format: Option<&String>,
        enabled: bool,
    );
    fn add_line(&self, log_id: &String, line: &String);
    fn add_lines(&self, log_id: &String, lines: &Vec<String>) -> Range<usize>;
    fn get_format(&self, log_id: &String) -> Option<String>;
    fn get_logs(&self) -> Vec<(bool, String, Option<String>)>;
    fn get_lines(&self, log_id: &String) -> Vec<String>;
    fn extract_lines(&self, log_id: &String) -> Vec<String>;
    fn get_total_lines(&self) -> usize;
}

pub struct InMemmoryLogStore {
    /// K: log_path -> V: lines
    raw_lines: RwLock<HashMap<String, Vec<String>>>,
    /// K: log_path -> V: format
    format: RwLock<HashMap<String, String>>,
    /// K: log_path -> V: enabled
    enabled: RwLock<HashMap<String, bool>>,
    /// K: log_path -> V: source controller
    source: RwLock<HashMap<String, Arc<Box<dyn LogSource + Send + Sync>>>>,
}

impl InMemmoryLogStore {
    pub fn new() -> Self {
        Self {
            raw_lines: RwLock::new(HashMap::default()),
            format: RwLock::new(HashMap::default()),
            enabled: RwLock::new(HashMap::default()),
            source: RwLock::new(HashMap::default()),
        }
    }
}

impl LogStore for InMemmoryLogStore {
    fn add_log(
        &self,
        log_id: &String,
        log_source: Arc<Box<dyn LogSource + Send + Sync>>,
        format: Option<&String>,
        enabled: bool,
    ) {
        let (mut source_lock, mut format_lock, mut enabled_lock) = (
            self.source.write(),
            self.format.write(),
            self.enabled.write(),
        );

        source_lock.insert(log_id.clone(), log_source);
        enabled_lock.insert(log_id.clone(), enabled);

        if let Some(format) = format {
            format_lock.insert(log_id.clone(), format.clone());
        }
    }

    fn add_line(&self, log_id: &String, line: &String) {
        let mut raw_lines_lock = self.raw_lines.write();

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.clone(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        raw_lines.push(line.clone());
    }

    fn add_lines(&self, log_id: &String, lines: &Vec<String>) -> Range<usize> {
        let mut raw_lines_lock = self.raw_lines.write();

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.clone(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        let current_len = raw_lines.len();
        raw_lines.append(&mut lines.clone());

        let new_len = raw_lines.len();
        current_len..new_len
    }

    fn get_lines(&self, log_id: &String) -> Vec<String> {
        match self.raw_lines.read().get(log_id) {
            Some(lines) => lines.clone(),
            _ => Vec::new(),
        }
    }

    fn extract_lines(&self, log_id: &String) -> Vec<String> {
        let mut w = self.raw_lines.write();
        let lines = std::mem::take(w.get_mut(log_id).unwrap());

        lines
    }

    fn get_logs(&self) -> Vec<(bool, String, Option<String>)> {
        let (format_lock, enabled_lock) =
            (self.format.read(), self.enabled.read());

        let logs: Vec<(bool, String, Option<String>)> = enabled_lock
            .iter()
            .map(|(path, enabled)| {
                (
                    enabled.clone(),
                    path.clone(),
                    match format_lock.get(path) {
                        Some(path) => Some(path.clone()),
                        _ => None,
                    },
                )
            })
            .collect();
        logs
    }

    fn get_format(&self, log_id: &String) -> Option<String> {
        let format_lock = self.format.read();
        match format_lock.get(log_id) {
            Some(alias) => Some(alias.clone()),
            _ => None,
        }
    }

    fn get_total_lines(&self) -> usize {
        self.raw_lines.read().values().fold(0, |acc, v| acc + v.len())
    }
}
