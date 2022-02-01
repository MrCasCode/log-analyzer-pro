use crate::models::log_line::LogLine;
use async_std::sync::RwLock;
use async_trait::async_trait;

#[async_trait]
pub trait AnalysisStore {
    async fn add_lines(&self, lines: &[&LogLine]);
    async fn add_search_lines(&self, lines: &[&LogLine]);
    async fn get_search_query(&self) -> Option<String>;
    async fn fetch_log(&self) -> Vec<LogLine>;
}
pub struct InMemmoryAnalysisStore {
    log: RwLock<Vec<LogLine>>,
    search_log: RwLock<(Option<String>, Vec<LogLine>)>
}

impl InMemmoryAnalysisStore {
    pub fn new() -> Self {
        Self {
            log : RwLock::new(Vec::new()),
            search_log : RwLock::new((None, Vec::new())),
        }
    }
}

#[async_trait]
impl AnalysisStore for InMemmoryAnalysisStore {
    async fn add_lines(&self, lines: &[&LogLine]) {
        let mut w = self.log.write().await;
        for &line in lines {
            w.push(line.clone());
        }
    }

    async fn add_search_lines(&self, lines: &[&LogLine]) {
        let mut w = self.search_log.write().await;
        for &line in lines {
            w.1.push(line.clone());
        }
    }

    async fn get_search_query(&self) -> Option<String> {
        let r = self.search_log.read().await;
        r.0.clone()
    }

    async fn fetch_log(&self) -> Vec<LogLine> {
        let r = self.log.read().await;
        r.clone()
    }
}