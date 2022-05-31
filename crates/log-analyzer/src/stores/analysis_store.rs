use crate::models::log_line::LogLine;
use parking_lot::{lock_api::RwLockReadGuard, RawRwLock, RwLock};

/// Store for managing processed logs.
///
/// Stores both the combined filtered log and the search log
pub trait AnalysisStore {
    /// Add a list of processed lines
    fn add_lines(&self, lines: &[LogLine]);
    /// Add a list of searched lines
    fn add_search_lines(&self, lines: &[LogLine]);
    /// Change the search query
    fn add_search_query(&self, query: &String);
    /// Get the current search query
    fn get_search_query(&self) -> Option<String>;
    /// Clear the processed log
    fn reset_log(&self);
    /// Clear the searched log
    fn reset_search(&self);
    /// Get a RwLock to the current processed log to avoid copying
    fn fetch_log(&self) -> RwLockReadGuard<RawRwLock, Vec<LogLine>>;
    /// Get a RwLock to the current searched log to avoid copying
    fn fetch_search(&self) -> RwLockReadGuard<RawRwLock, Vec<LogLine>>;
    /// Get a copy of a window of lines. Is safe to query out of bounds
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    /// Get a copy of a window of search lines. Is safe to query out of bounds
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    /// Get a window of `elements` number of lines centered around the target `line`
    ///
    /// Returns (list of lines, offset from start, index of target)
    fn get_log_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);
    /// Get a window of `elements` number of lines centered around the target `line`
    ///
    /// Returns (list of lines, offset from start, index of target)
    fn get_search_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);
    /// Count the total number of lines
    fn get_total_filtered_lines(&self) -> usize;
    /// Count the total number of search lines
    fn get_total_searched_lines(&self) -> usize;
}
pub struct InMemmoryAnalysisStore {
    log: RwLock<Vec<LogLine>>,
    search_query: RwLock<Option<String>>,
    search_log: RwLock<Vec<LogLine>>,
}

impl InMemmoryAnalysisStore {
    pub fn new() -> Self {
        Self {
            log: RwLock::new(Vec::new()),
            search_query: RwLock::new(None),
            search_log: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemmoryAnalysisStore {
    fn default() -> Self {
        Self::new()
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

    fn fetch_log(&self) -> RwLockReadGuard<RawRwLock, Vec<LogLine>> {
        self.log.read()
    }

    fn fetch_search(&self) -> RwLockReadGuard<RawRwLock, Vec<LogLine>> {
        self.search_log.read()
    }

    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.log.read();
        log[from.min(log.len())..to.min(log.len())].to_vec()
    }

    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        let log = self.search_log.read();
        log[from.min(log.len())..to.min(log.len())].to_vec()
    }

    fn get_log_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        let log = self.log.read();
        InMemmoryAnalysisStore::find_rolling_window(&log, line, elements)
    }

    fn get_search_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
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
        match source.binary_search_by(|e| {
            e.index
                .parse::<usize>()
                .unwrap()
                .cmp(&element.index.parse::<usize>().unwrap())
        }) {
            Ok(i) => i,
            Err(i) => i,
        }
    }

    /// Find a window of elements containing the target in the middle
    /// Returns (elements, offset, index)
    fn find_rolling_window(
        source: &[LogLine],
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        let closest = InMemmoryAnalysisStore::find_sorted_index(source, &line);
        let from = if (elements / 2) < closest {
            closest - elements / 2
        } else {
            0
        };
        let to = (closest + elements / 2).min(source.len());

        let lines = source[from..to].to_vec();
        let index = InMemmoryAnalysisStore::find_sorted_index(&lines, &line);
        (lines, from, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_line_with_index(index: usize) -> LogLine {
        LogLine {
            index: index.to_string(),
            ..Default::default()
        }
    }
}
