use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use async_std::{channel, channel::Receiver, channel::Sender, prelude::*};
//use tokio::sync::{broadcast, broadcast::Receiver, broadcast::Sender};

use futures::join;
use regex::Regex;

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
        &mut self,
        source_type: usize,
        source_address: &String,
        format: &String,
    ) -> Result<()>;
    async fn add_format(&self, alias: &String, regex: &String) -> Result<()>;
    fn add_search(&self, regex: &String) -> Result<()>;
    fn get_log(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_search(&self) -> Arc<RwLock<Vec<LogLine>>>;
    fn get_logs(&self) -> Vec<(bool, String, String)>;
    fn get_formats(&self) -> Vec<Format>;
    fn get_filters(&self) -> Vec<Filter>;
    fn on_event(&self) -> Receiver<Event>;
}

pub struct LogService {
    log_store: Arc<dyn LogStore + Sync + Send>,
    processing_store: Arc<dyn ProcessingStore + Sync + Send>,
    analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    source_channels: (Sender<(String, String)>, Receiver<(String, String)>),
    format_channels: (Sender<(String, String)>, Receiver<(String, String)>),
    filter_channels: (Sender<(String, LogLine)>, Receiver<(String, LogLine)>),
    event_channels: (Sender<Event>, Receiver<Event>),
}

impl LogService {
    pub fn new(
        log_store: Arc<dyn LogStore + Sync + Send>,
        processing_store: Arc<dyn ProcessingStore + Sync + Send>,
        analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    ) -> Self {
        let source_channels = channel::unbounded();
        let format_channels = channel::unbounded();
        let filter_channels = channel::unbounded();
        let event_channels = channel::unbounded();

        let raw_line_log_store = log_store.clone();
        let raw_line_receiver = source_channels.1.clone();
        let raw_line_format_sender = format_channels.0.clone();
        std::thread::spawn(|| {
            async_std::task::spawn(async move {
                while let Ok((path, line)) = raw_line_receiver.recv().await {
                    raw_line_log_store.add_line(&path, &line);
                    if let Some(alias) = raw_line_log_store.get_format(&path) {
                        raw_line_format_sender.send((alias, line)).await;
                    }
                }
            });
        });

        let format_line_processing_store = processing_store.clone();
        let format_line_receiver = format_channels.1.clone();
        let format_line_filter_sender = filter_channels.0.clone();
        std::thread::spawn(|| {
            async_std::task::spawn(async move {
                while let Ok((path, line)) = format_line_receiver.recv().await {
                    if let Some(format) = format_line_processing_store.get_format(&path) {
                        if let Some(line) = apply_format(&format, &line) {
                            format_line_filter_sender.send((path, line)).await;
                        }
                    }
                }
            });
        });

        let filter_line_processing_store = processing_store.clone();
        let filter_line_analysis_store = analysis_store.clone();
        let filter_line_receiver = filter_channels.1.clone();
        let filter_line_sender = event_channels.0.clone();

        std::thread::spawn(|| {
            async_std::task::spawn(async move {
                while let Ok((_path, log_line)) = filter_line_receiver.recv().await {
                    let filters = filter_line_processing_store.get_filters();
                    if let Some(filtered_line) = apply_filters(&filters, log_line) {
                        let search_query = filter_line_analysis_store.get_search_query();
                        filter_line_analysis_store.add_lines(&[&filtered_line]);

                        filter_line_sender.send(Event::NewLine).await;

                        if search_query.is_some()
                            && apply_search(&search_query.unwrap(), &filtered_line)
                        {
                            filter_line_analysis_store.add_search_lines(&[&filtered_line]);

                            filter_line_sender.send(Event::NewSearchLine).await;
                        }
                    }
                }
            });
        });

        Self {
            log_store,
            processing_store,
            analysis_store,
            source_channels,
            format_channels,
            filter_channels,
            event_channels,
        }
    }
}

#[async_trait]
impl LogAnalyzer for LogService {
    async fn add_log(
        &mut self,
        source_type: usize,
        source_address: &String,
        format: &String,
    ) -> Result<()> {
        let sender = self.source_channels.0.clone();
        let log_store = self.log_store.clone();

        let source_type = SourceType::try_from(source_type).unwrap();

        let log_source = Arc::new(create_source(source_type, source_address.clone()).await?);
        log_store.add_log(&source_address, log_source.clone(), &format, true);

        async_std::task::spawn(async move {
            log_source.run(sender).await.unwrap();
        });

        Ok(())
    }

    async fn add_format(&self, alias: &String, regex: &String) -> Result<()> {
        let format = Format::new(alias, regex)?;
        self.processing_store
            .add_format(format.alias, format.regex);
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

    fn get_filters(&self) -> Vec<Filter> {
        self.processing_store.get_filters()
    }

    fn on_event(&self) -> Receiver<Event> {
        self.event_channels.1.clone()
    }
}
