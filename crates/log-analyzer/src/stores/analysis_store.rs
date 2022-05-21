use rayon::slice::ParallelSliceMut;

use crate::models::log_line::LogLine;
use std::sync::{Arc, RwLock};

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
        let mut w = self.log.write().unwrap();
        for &line in lines {
            w.push(line.clone());
        }
        //w.par_sort_unstable_by(|a, b| a.timestamp.trim_start().parse::<f64>().unwrap().partial_cmp(&b.timestamp.trim_start().parse::<f64>().unwrap()).unwrap());
    }

    fn add_search_lines(&self, lines: &[&LogLine]) {
        let mut w = self.search_log.write().unwrap();
        for &line in lines {
            w.push(line.clone());
        }

        w.par_sort_unstable_by(|a, b| a.index.trim_start().parse::<usize>().unwrap().partial_cmp(&b.index.trim_start().parse::<usize>().unwrap()).unwrap());

    }

    fn add_search_query(&self, query: &String) {
        let mut w = self.search_query.write().unwrap();
        *w = Some(query.clone());
    }

    fn get_search_query(&self) -> Option<String> {
        let r = self.search_query.read().unwrap();
        r.clone()
    }

    fn fetch_log(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.log.clone()
    }

    fn fetch_search(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.search_log.clone()
    }

    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.log.read().unwrap();
        log[from.min(log.len())..to.min(log.len())].to_vec()
    }

    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.search_log.read().unwrap();
        log[from.min(log.len())..to.min(log.len())].to_vec()

    }
    fn reset_log(&self) {
        let mut w = self.log.write().unwrap();
        w.clear();
    }

    fn reset_search(&self) {
        let mut w = self.search_log.write().unwrap();
        w.clear();
    }

    fn get_total_filtered_lines(&self) -> usize {
        self.log.read().unwrap().len()
    }

    fn get_total_searched_lines(&self) -> usize {
        self.search_log.read().unwrap().len()
    }
}
