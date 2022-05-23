use crate::models::{log_line::LogLine};
use std::sync::{Arc};
use parking_lot::RwLock;

pub trait AnalysisStore {
    fn add_lines(&self, lines: &[LogLine]);
    fn add_search_lines(&self, lines: &[LogLine]);
    fn add_search_query(&self, query: &String);
    fn get_search_query(&self) -> Option<String>;
    fn reset_log(&self);
    fn reset_search(&self);
    fn fetch_log(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn fetch_search(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_log_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize, usize);
    fn get_search_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize, usize);
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
    fn add_lines(&self, lines: &[LogLine]) {
        let mut w = self.log.write();
        for line in lines {
            w.push(line.clone());
        }
    }

    fn add_search_lines(&self, lines: &[LogLine]) {
        let mut w = self.search_log.write();
        for line in lines {
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

    fn get_log_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize, usize) {
        let log = self.log.read();
        InMemmoryAnalysisStore::find_rolling_window(&log, line, elements)

    }

    fn get_search_lines_containing(&self, line: LogLine, elements: usize) -> (Vec<LogLine>, usize, usize) {
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

    fn find_sorted_index(source: &[LogLine], element: &LogLine) -> usize {
        match source.binary_search_by(|e| e.index.parse::<usize>().unwrap().cmp(&element.index.parse::<usize>().unwrap())) {
            Ok(i) => i,
            Err(i) => i,
        }
    }

    /// Find a window of elements containing the target in the middle
    /// Returns (elements, offset, index)
    fn find_rolling_window(source: &[LogLine], line: LogLine, elements: usize) -> (Vec<LogLine>, usize, usize) {
        let closest = InMemmoryAnalysisStore::find_sorted_index(source, &line);
        let from = if (elements / 2) < closest {closest - elements / 2} else {0};
        let to = (closest + elements / 2).min(source.len());

        let lines = source[from..to].to_vec();
        let index = InMemmoryAnalysisStore::find_sorted_index(&lines, &line);
        (lines, from, index)
    }

    fn append_sorted_chunk(v: &mut Vec<LogLine>, new_data: &[LogLine]) {
        if let Some(first) = new_data.first() {
            let index = InMemmoryAnalysisStore::find_sorted_index(&v, first);
            let (a, b) = v.split_at(index);
            *v = [a, new_data, b].concat();
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    fn log_line_with_index(index: usize) -> LogLine {
        LogLine { index: index.to_string(), date: "".to_string(), timestamp: "".to_string(), app: "".to_string(), severity: "".to_string(), function: "".to_string(), payload: "".to_string(), color: None }
    }

    #[test]
    fn test_append_sorted_chunk_at_end() {
        let mut current_lines: Vec<LogLine> = (0..100).into_iter().map(|index: usize| log_line_with_index(index)).collect();
        let new_lines: Vec<LogLine> = (100..200).into_iter().map(|index: usize| log_line_with_index(index)).collect();

        InMemmoryAnalysisStore::append_sorted_chunk(&mut current_lines, &new_lines);

        assert!(current_lines[100].index == "100")
    }


    #[test]
    fn test_append_sorted_chunk_at_mid() {
        let mut current_lines: Vec<LogLine> = (0..80).into_iter().map(|index: usize| log_line_with_index(index)).collect();

        let mut new_lines: Vec<LogLine> = (200..300).into_iter().map(|index: usize| log_line_with_index(index)).collect();
        InMemmoryAnalysisStore::append_sorted_chunk(&mut current_lines, &new_lines);

        new_lines = (100..200).into_iter().map(|index: usize| log_line_with_index(index)).collect();
        InMemmoryAnalysisStore::append_sorted_chunk(&mut current_lines, &new_lines);

        new_lines = (80..100).into_iter().map(|index: usize| log_line_with_index(index)).collect();
        InMemmoryAnalysisStore::append_sorted_chunk(&mut current_lines, &new_lines);

        assert!(current_lines[100].index == "100")
    }
}
