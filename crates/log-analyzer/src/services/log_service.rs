use std::ops::Range;
use std::sync::Arc;

use anyhow::Result;
use flume::{Sender};
use tokio::sync::broadcast;
use regex::Regex;

use pariter::{scope, IteratorExt as _};

use crate::domain::apply_filters::apply_filters;
use crate::domain::apply_format::apply_format;
use crate::domain::apply_search::apply_search;
use crate::models::{filter::Filter, format::Format, log_line::LogLine};
use crate::stores::analysis_store::AnalysisStore;
use crate::stores::log_store::LogStore;
use crate::stores::processing_store::ProcessingStore;

use super::log_source::{create_source, SourceType};

use async_trait::async_trait;

#[derive(Clone, PartialEq)]
pub enum Event {
    Processing(usize, usize),
    NewLines(usize, usize),
    NewSearchLines(usize, usize),
    Filtering,
    FilterFinished,
    Searching,
    SearchFinished,
}

#[async_trait]
pub trait LogAnalyzer {
    async fn add_log(
        &self,
        source_type: usize,
        source_address: &String,
        format: Option<&String>,
    ) -> Result<()>;
    fn add_format(&self, alias: &String, regex: &String) -> Result<()>;
    fn add_search(&self, regex: &String);
    fn add_filter(&self, filter: Filter);
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    fn get_log_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);
    fn get_search_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);
    fn get_logs(&self) -> Vec<(bool, String, Option<String>)>;
    fn get_formats(&self) -> Vec<Format>;
    fn get_filters(&self) -> Vec<(bool, Filter)>;
    fn get_total_raw_lines(&self) -> usize;
    fn get_total_filtered_lines(&self) -> usize;
    fn get_total_searched_lines(&self) -> usize;
    fn toggle_filter(&self, id: &String);
    fn on_event(&self) -> broadcast::Receiver<Event>;
}

pub struct LogService {
    log_store: Arc<dyn LogStore + Sync + Send>,
    processing_store: Arc<dyn ProcessingStore + Sync + Send>,
    analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    log_sender: Sender<(String, Vec<String>)>,
    event_channel: broadcast::Sender<Event>,
}

impl LogService {
    pub fn new(
        log_store: Arc<dyn LogStore + Sync + Send>,
        processing_store: Arc<dyn ProcessingStore + Sync + Send>,
        analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    ) -> Arc<Self> {
        let (sender, receiver) = flume::bounded(1_000_000_usize);
        let (broadcast_sender, _broadcast_receiver) = broadcast::channel(100);

        let log_service = Arc::new(Self {
            log_store,
            processing_store,
            analysis_store,
            log_sender: sender,
            event_channel: broadcast_sender,
        });

        let log = log_service.clone();
        let event_sender = log_service.event_channel.clone();
        std::thread::Builder::new()
            .name("Consumer".to_string())
            .spawn(move || loop {
                let num_cpus = num_cpus::get();
                while let Ok((path, lines)) = receiver.recv() {
                    let (format, indexes, lines) = log.process_raw_lines(path, lines);

                    if !lines.is_empty() {
                        let chunk_size = lines.len() / num_cpus;

                        let elements: Vec<(String, usize)> = lines
                            .into_iter()
                            .zip(indexes)
                            .map(|(line, index)| (line, index))
                            .collect();

                        let first_index = elements[0].1;
                        let last_index = elements.last().unwrap().1;
                        event_sender
                            .send(Event::Processing(first_index, last_index))
                            .unwrap_or_default();

                        scope(|scope| {
                            let processed: Vec<(Vec<LogLine>, Vec<LogLine>)> = elements
                                .chunks(chunk_size.max(num_cpus))
                                .parallel_map_scoped(scope, |chunk| {
                                    let lines = log.apply_format(&format, chunk);
                                    let filtered_lines = log.apply_filters(lines);
                                    let (filtered, search) = log.apply_search(filtered_lines);
                                    (filtered, search)
                                })
                                .collect();

                            for (filtered, search) in processed {
                                log.analysis_store.add_lines(&filtered);
                                log.analysis_store.add_search_lines(&search);
                            }

                            event_sender
                                .send(Event::NewLines(first_index, last_index))
                                .unwrap_or_default();
                            event_sender
                                .send(Event::NewSearchLines(first_index, last_index))
                                .unwrap_or_default();
                        })
                        .unwrap();
                    }
                }
            })
            .unwrap();

        log_service
    }

    fn process_raw_lines(
        &self,
        path: String,
        lines: Vec<String>,
    ) -> (Option<String>, Range<usize>, Vec<String>) {
        let indexes = self.log_store.add_lines(&path, &lines);
        let format = self.log_store.get_format(&path);
        (format, indexes, lines)
    }

    fn apply_format(
        &self,
        format: &Option<String>,
        line_index: &[(String, usize)],
    ) -> Vec<LogLine> {
        let mut format_regex = None;

        if let Some(format) = format {
            let format = self.processing_store.get_format(format);
            format_regex = match format {
                Some(format) => Some(Regex::new(&format).unwrap()),
                _ => None,
            };
        }

        let mut log_lines: Vec<LogLine> = Vec::with_capacity(line_index.len());
        for (line, index) in line_index {
            let log_line = apply_format(&format_regex.as_ref(), &line, *index);
            log_lines.push(log_line);
        }
        log_lines
    }

    fn apply_filters(&self, lines: Vec<LogLine>) -> Vec<LogLine> {
        let filters: Vec<Filter> = self
            .processing_store
            .get_filters()
            .into_iter()
            .filter(|(enabled, _)| *enabled)
            .map(|(_, filter)| filter)
            .collect();

        let mut filtered_lines: Vec<LogLine> = Vec::with_capacity(lines.len());

        for line in lines {
            if let Some(filtered_line) = apply_filters(&filters, line) {
                filtered_lines.push(filtered_line);
            }
        }
        filtered_lines
    }

    fn apply_search(&self, lines: Vec<LogLine>) -> (Vec<LogLine>, Vec<LogLine>) {
        let mut search_lines: Vec<LogLine> = Vec::with_capacity(lines.len());
        if let Some(search_query) = self.analysis_store.get_search_query() {
            if let Ok(search_regex) = Regex::new(&search_query) {
                for line in &lines {
                    if apply_search(&search_regex, &line) {
                        search_lines.push(line.clone());
                    }
                }
            }
        }

        (lines, search_lines)
    }
}

#[async_trait]
impl LogAnalyzer for LogService {
    async fn add_log(
        &self,
        source_type: usize,
        source_address: &String,
        format: Option<&String>,
    ) -> Result<()> {
        let sender = self.log_sender.clone();
        let log_store = self.log_store.clone();

        let source_type = SourceType::try_from(source_type).unwrap();

        let log_source = Arc::new(create_source(source_type, source_address.clone()).await?);
        log_store.add_log(&source_address, log_source.clone(), format, true);

        std::thread::Builder::new()
            .name(source_address.clone())
            .spawn(|| {
                async_std::task::spawn(async move {
                    log_source.run(sender).await.unwrap();
                });
            })
            .unwrap();

        Ok(())
    }

    fn add_format(&self, alias: &String, regex: &String) -> Result<()> {
        let format = Format::new(alias, regex)?;

        self.processing_store.add_format(format.alias, format.regex);
        Ok(())
    }

    fn add_search(&self, regex: &String) {
        let re = Regex::new(&regex);
        self.analysis_store.reset_search();
        self.event_channel
            .send(Event::Searching)
            .unwrap_or_default();

        match re {
            Ok(_) => {
                self.analysis_store.add_search_query(regex);

                if let Ok(_) = Regex::new(&regex) {
                    let analysis_store = self.analysis_store.clone();
                    let regex_str = regex.clone();
                    let sender = self.event_channel.clone();

                    std::thread::Builder::new()
                        .name("Search".to_string())
                        .spawn(move || {
                            let r_lock = analysis_store.fetch_log();
                            let log = r_lock.read();
                            scope(|scope| {
                                let num_cpus = num_cpus::get();
                                let chunk_size = log.len() / num_cpus;
                                let search_lines: Vec<LogLine> = log
                                    .chunks(chunk_size.max(num_cpus))
                                    .parallel_map_scoped(scope, move |chunk| {
                                        let lines = chunk.to_owned();
                                        let r = Regex::new(&regex_str).unwrap();
                                        let mut v: Vec<LogLine> = Vec::with_capacity(lines.len());

                                        for log_line in lines {
                                            if apply_search(&r, &log_line) {
                                                v.push(log_line);
                                            };
                                        }

                                        v
                                    })
                                    .flatten()
                                    .collect::<Vec<LogLine>>();
                                analysis_store.add_search_lines(&search_lines);
                                sender.send(Event::SearchFinished).unwrap_or_default();
                            })
                            .unwrap();
                        })
                        .unwrap();

                }
            }
            Err(_) => {}
        }
    }

    fn add_filter(&self, filter: Filter) {
        self.processing_store
            .add_filter(filter.alias, filter.filter, filter.action, false);
    }

    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        self.analysis_store.get_log_lines(from, to)
    }

    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine> {
        self.analysis_store.get_search_lines(from, to)
    }

    fn get_log_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        self.analysis_store.get_log_lines_containing(line, elements)
    }

    fn get_search_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        self.analysis_store
            .get_search_lines_containing(line, elements)
    }

    fn get_logs(&self) -> Vec<(bool, String, Option<String>)> {
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
        self.analysis_store.reset_search();

        self.event_channel
            .send(Event::Filtering)
            .unwrap_or_default();

        let enabled_logs: Vec<String> = self
            .log_store
            .get_logs()
            .into_iter()
            .filter(|(enabled, _, _)| *enabled)
            .map(|(_, id, _)| id)
            .collect();

        let log_store = self.log_store.clone();
        let sender = self.log_sender.clone();

        let t = std::thread::Builder::new()
            .name("Toggle filter".to_string())
            .spawn(move || {
                for log in enabled_logs {
                    let lines = log_store.extract_lines(&log);
                    lines.chunks(1_000_000_usize).for_each(|lines| {
                        match sender.send((log.clone(), lines.to_vec())) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    });
                }
            })
            .unwrap();

        t.join().unwrap();

        let ev = async_std::task::block_on(self.event_channel.subscribe().recv()).unwrap();
        if matches!(ev,  Event::NewLines(_, _)) {
            self.event_channel
                .send(Event::FilterFinished)
                .unwrap_or_default();
        }
    }

    fn get_total_raw_lines(&self) -> usize {
        self.log_store.get_total_lines()
    }

    fn get_total_filtered_lines(&self) -> usize {
        self.analysis_store.get_total_filtered_lines()
    }

    fn get_total_searched_lines(&self) -> usize {
        self.analysis_store.get_total_searched_lines()
    }

    fn on_event(&self) -> broadcast::Receiver<Event> {
        self.event_channel.subscribe()
    }
}
