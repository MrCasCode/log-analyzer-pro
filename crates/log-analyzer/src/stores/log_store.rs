use log_source::source::log_source::LogSource;
use parking_lot::RwLock;
use rustc_hash::FxHashMap as HashMap;
use std::{iter::Iterator, ops::Range, sync::Arc};

/// Store holding raw information
///
/// Manages raw lines and associated format
pub trait LogStore {
    /// Add a new log to the store
    fn add_log(
        &self,
        log_id: &str,
        log_source: Arc<Box<dyn LogSource + Send + Sync>>,
        format: Option<&String>,
        enabled: bool,
    );
    /// Add a single line to the given log id
    fn add_line(&self, log_id: &str, line: &str);
    /// Add a many lines to the given log id
    fn add_lines(&self, log_id: &str, lines: &[String]) -> Range<usize>;
    /// Get the format associated to the given log id
    fn get_format(&self, log_id: &str) -> Option<String>;
    /// Get a list of (enabled, log_id, format(if any))
    fn get_logs(&self) -> Vec<(bool, String, Option<String>)>;
    /// Get the log source associated to the log id
    fn get_source(&self, id: &str) -> Option<Arc<Box<dyn LogSource + Send + Sync>>>;
    /// Get a list of all the lines for the requested log. WARNING: clones
    fn get_lines(&self, log_id: &str) -> Vec<String>;
    /// Get a list of all the lines for the requested log. WARNING: moves
    fn extract_lines(&self, log_id: &str) -> Vec<String>;
    /// Get the count of all the lines
    fn get_total_lines(&self) -> usize;
    /// Change the enabled state of the given log
    fn toggle_log(&self, log_id: &str);
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

impl Default for InMemmoryLogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LogStore for InMemmoryLogStore {
    fn add_log(
        &self,
        log_id: &str,
        log_source: Arc<Box<dyn LogSource + Send + Sync>>,
        format: Option<&String>,
        enabled: bool,
    ) {
        let (mut source_lock, mut format_lock, mut enabled_lock) = (
            self.source.write(),
            self.format.write(),
            self.enabled.write(),
        );

        source_lock.insert(log_id.to_string(), log_source);
        enabled_lock.insert(log_id.to_string(), enabled);

        if let Some(format) = format {
            format_lock.insert(log_id.to_string(), format.clone());
        }
    }

    fn add_line(&self, log_id: &str, line: &str) {
        let mut raw_lines_lock = self.raw_lines.write();

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.to_string(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        raw_lines.push(line.to_string());
    }

    fn add_lines(&self, log_id: &str, lines: &[String]) -> Range<usize> {
        let mut raw_lines_lock = self.raw_lines.write();

        if !raw_lines_lock.contains_key(log_id) {
            raw_lines_lock.insert(log_id.to_string(), Vec::new());
        }
        let raw_lines = raw_lines_lock.get_mut(log_id).unwrap();
        let current_len = raw_lines.len();
        raw_lines.append(&mut lines.to_vec());

        let new_len = raw_lines.len();
        current_len..new_len
    }

    fn get_lines(&self, log_id: &str) -> Vec<String> {
        match self.raw_lines.read().get(log_id) {
            Some(lines) => lines.clone(),
            _ => Vec::new(),
        }
    }

    fn extract_lines(&self, log_id: &str) -> Vec<String> {
        let mut w = self.raw_lines.write();
        let lines = std::mem::take(w.get_mut(log_id).unwrap());

        lines
    }

    fn get_logs(&self) -> Vec<(bool, String, Option<String>)> {
        let (format_lock, enabled_lock) = (self.format.read(), self.enabled.read());

        let logs: Vec<(bool, String, Option<String>)> = enabled_lock
            .iter()
            .map(|(path, enabled)| (*enabled, path.clone(), format_lock.get(path).cloned()))
            .collect();
        logs
    }

    fn get_format(&self, log_id: &str) -> Option<String> {
        let format_lock = self.format.read();
        format_lock.get(log_id).cloned()
    }

    fn get_total_lines(&self) -> usize {
        self.raw_lines
            .read()
            .values()
            .fold(0, |acc, v| acc + v.len())
    }

    fn get_source(&self, id: &str) -> Option<Arc<Box<dyn LogSource + Send + Sync>>> {
        if let Some((_id, source)) = self
            .source
            .read()
            .iter()
            .find(|(log_id, _source)| *id == **log_id)
        {
            Some(source.clone())
        } else {
            None
        }
    }

    fn toggle_log(&self, log_id: &str) {
        if let Some(e) = self.enabled.write().get_mut(log_id) {
            *e = !*e;
        }
    }
}
