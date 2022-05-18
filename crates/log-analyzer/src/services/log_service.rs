use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use std::time::Duration;
use std::sync::mpsc::{self, SyncSender};
use std::sync::mpsc::{Receiver, Sender};
//use tokio::sync::{broadcast, broadcast::Receiver, broadcast::Sender};

use futures::join;
use regex::Regex;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;

use crate::domain::apply_filters::apply_filters;
use crate::domain::apply_format::apply_format;
use crate::domain::apply_search::apply_search;
use crate::models::filter::FilterAction;
use crate::models::{filter::Filter, format::Format, log::Log, log_line::LogLine};
use crate::stores::analysis_store::AnalysisStore;
use crate::stores::log_store::LogStore;
use crate::stores::processing_store::ProcessingStore;

use super::log_source::{create_source, LogSource, SourceType};

use async_trait::async_trait;

#[derive(Clone, PartialEq)]
pub enum Event {
    NewLine,
    NewSearchLine,
}

#[async_trait]
pub trait LogAnalyzer {
    async fn add_log(
        &self,
        source_type: usize,
        source_address: &String,
        format: &String,
    ) -> Result<()>;
    fn add_format(&self, alias: &String, regex: &String) -> Result<()>;
    fn add_search(&self, regex: &String) -> Result<()>;
    fn get_log(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_search(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_logs(&self) -> Vec<(bool, String, String)>;
    fn get_formats(&self) -> Vec<Format>;
    fn get_filters(&self) -> Vec<(bool, Filter)>;
    fn toggle_filter(&self, id: &String);
}

pub struct LogService {
    log_store: Arc<dyn LogStore + Sync + Send>,
    processing_store: Arc<dyn ProcessingStore + Sync + Send>,
    analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    sender: SyncSender<(String, String)>,
}

impl LogService {
    pub fn new(
        log_store: Arc<dyn LogStore + Sync + Send>,
        processing_store: Arc<dyn ProcessingStore + Sync + Send>,
        analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    ) -> Arc<Self> {
        let (sender, receiver) = mpsc::sync_channel(4096);

        let log_service = Arc::new(Self {
            log_store,
            processing_store,
            analysis_store,
            sender,
        });

        let log = log_service.clone();
        std::thread::spawn(move || {
            let timeout = Duration::from_millis(10);
            loop {
                let mut processing_queue = Vec::with_capacity(100);
                while let Ok(value) = receiver.recv_timeout(timeout) {
                    processing_queue.push(value);

                    if processing_queue.len() >= 100 {
                        break;
                    }
                }
                processing_queue
                  .into_par_iter()
                  .filter_map(|(path, line)| log.process_raw_line(path, line))
                  .filter_map(|(path, line)| log.apply_format(path, line))
                  .filter_map(|(path, line)| log.apply_filters(path, line))
                  .for_each(|(path, line)| log.apply_search(path, line));
            }
        });

        log_service
    }

    fn process_raw_line(&self, path: String, line: String) -> Option<(String, String)> {
        self.log_store.add_line(&path, &line);
        match self.log_store.get_format(&path) {
            Some(alias) => Some((alias, line)),
            None => None,
        }
    }

    fn apply_format(&self, path: String, line: String) -> Option<(String, LogLine)> {
        let format = self.processing_store.get_format(&path)?;

        match apply_format(&format, &line) {
            Some(line) => Some((path, line)),
            None => None,
        }
    }

    fn apply_filters(&self, path: String, log_line: LogLine) -> Option<(String, LogLine)>{
        let filters: Vec<Filter> = self.processing_store
                        .get_filters()
                        .into_iter()
                        .filter(|(enabled, _)| *enabled)
                        .map(|(_, filter)| filter)
                        .collect();

        let filtered_line = apply_filters(&filters, log_line)?;
        self.analysis_store.add_lines(&[&filtered_line]);
        Some((path, filtered_line))
    }

    fn apply_search(&self, path: String, log_line: LogLine) {
        let search_query = self.analysis_store.get_search_query();

        if search_query.is_some() && apply_search(&search_query.unwrap(), &log_line)
        {
            self.analysis_store.add_search_lines(&[&log_line]);
        }
    }
}

#[async_trait]
impl LogAnalyzer for LogService {
    async fn add_log(
        &self,
        source_type: usize,
        source_address: &String,
        format: &String,
    ) -> Result<()> {
        let sender = self.sender.clone();
        let log_store = self.log_store.clone();

        let source_type = SourceType::try_from(source_type).unwrap();

        let log_source = Arc::new(create_source(source_type, source_address.clone()).await?);
        log_store.add_log(&source_address, log_source.clone(), &format, true);

        std::thread::spawn(|| {
            async_std::task::spawn(async move {
                log_source.run(sender).await.unwrap();
            });
        });

        Ok(())
    }

    fn add_format(&self, alias: &String, regex: &String) -> Result<()> {
        let format = Format::new(alias, regex)?;
        self.processing_store.add_format(format.alias, format.regex);
        Ok(())
    }

    fn add_search(&self, regex: &String) -> Result<()> {
        let re = Regex::new(&regex);
        match re {
            Ok(r) => {
                self.analysis_store.add_search_query(regex);
                self.analysis_store.reset_search();

                let r = self.analysis_store.fetch_log();
                let log = r.read().unwrap();

                for log_line in &*log {
                    if apply_search(&regex, &log_line) {
                        self.analysis_store.add_search_lines(&[&log_line]);
                    }
                }

                Ok(())
            }
            Err(_) => Err(anyhow!(
                "Could not compile regex.\nPlease review regex syntax"
            )),
        }
    }

    fn get_log(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.analysis_store.fetch_log()
    }

    fn get_search(&self) -> Arc<RwLock<Vec<LogLine>>> {
        self.analysis_store.fetch_search()
    }

    fn get_logs(&self) -> Vec<(bool, String, String)> {
        self.log_store.get_logs()
    }

    fn get_formats(&self) -> Vec<Format> {
        self.processing_store.get_formats()
    }

    fn get_filters(&self) -> Vec<(bool, Filter)> {
        self.processing_store.get_filters()
    }

    fn toggle_filter(&self, id: &String) {
        self.processing_store.toggle_filter(id);
        self.analysis_store.reset_log();

        let enabled_logs: Vec<String> = self
            .log_store
            .get_logs()
            .into_iter()
            .filter(|(enabled, _, _)| *enabled)
            .map(|(_, id, _)| id)
            .collect();

        for log in enabled_logs {
            let lines = self.log_store.extract_lines(&log);
            for line in lines {
                self.sender.send((log.clone(), line));
            }
        }
    }
}
