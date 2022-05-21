use rayon::slice::{ParallelSliceMut, ParallelSlice};

use crate::models::{log_line::LogLine};
use std::sync::{Arc};
use parking_lot::RwLock;

pub trait AnalysisStore {
    fn add_lines(&self, lines: &[&LogLine]);
    fn add_search_lines(&self, lines: &[&LogLine]);
    fn add_search_query(&self, query: &String);
    fn get_search_query(&self) -> Option<String>;
    fn reset_log(&self);
    fn reset_search(&self);
    fn fetch_log(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn fetch_search(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_log_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize);
    fn get_search_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize);
    fn get_total_filtered_lines(&self) -> usize;
    fn get_total_searched_lines(&self) -> usize;
}
pub struct InMemmoryAnalysisStore {
    log: Arc<RwLock<Vec<LogLine>>>,
    search_query: Arc<RwLock<Option<String>>>,
    search_log: Arc<RwLock<Vec<LogLine>>>,
}

impl InMemmoryAnalysisStore {
    pub fn new() -> Self {
        Self {
            log: Arc::new(RwLock::new(Vec::new())),
            search_query: Arc::new(RwLock::new(None)),
            search_log: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl AnalysisStore for InMemmoryAnalysisStore {
    fn add_lines(&self, lines: &[&LogLine]) {
        let mut w = self.log.write();
        for &line in lines {
            w.push(line.clone());
        }
    }

    fn add_search_lines(&self, lines: &[&LogLine]) {
        let mut w = self.search_log.write();
        for &line in lines {
            w.push(line.clone());
        }
    }

    fn add_search_query(&self, query: &String) {
        let mut w = self.search_query.write();
        *w = Some(query.clone());
    }

    fn get_search_query(&self) -> Option<String> {
        let r = self.search_query.read();
        r.clone()
    }

    fn fetch_log(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.log.clone()
    }

    fn fetch_search(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.search_log.clone()
    }

    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.log.read();
        log[from.min(log.len())..to.min(log.len())].to_vec()
    }

    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.search_log.read();
        log[from.min(log.len())..to.min(log.len())].to_vec()
    }

    fn get_log_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize) {
        let log = self.log.read();
        InMemmoryAnalysisStore::find_rolling_window(&log, line, elements)

    }

    fn get_search_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize) {
        let search_log = self.search_log.read();
        InMemmoryAnalysisStore::find_rolling_window(&search_log, line, elements)

    }

    fn reset_log(&self) {
        let mut w = self.log.write();
        w.clear();
    }

    fn reset_search(&self) {
        let mut w = self.search_log.write();
        w.clear();
    }

    fn get_total_filtered_lines(&self) -> usize {
        self.log.read().len()
    }

    fn get_total_searched_lines(&self) -> usize {
        self.search_log.read().len()
    }
}


impl InMemmoryAnalysisStore {
    fn find_rolling_window(source: &[LogLine], line: LogLine, elements: usize) -> (Vec<LogLine>, usize) {
        let closest = match source.binary_search_by(|e| line.index.cmp(&e.index)) {
            Ok(i) => i,
            Err(i) => i,
        };
        let from = if (elements / 2) < closest {closest - elements / 2} else {0};
        let to = (closest + elements / 2).min(source.len());

        let lines = source[from..to].to_vec();
        let index = match source.binary_search_by(|e| line.index.cmp(&e.index)) {
            Ok(i) => i,
            Err(i) => i,
        };
        (lines, index)
    }
}
