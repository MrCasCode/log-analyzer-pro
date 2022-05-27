use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use flume::Sender;
use log_source::source::log_source::{create_source, SourceType};
use regex::Regex;
use tokio::sync::broadcast;

use pariter::{scope, IteratorExt as _};

use crate::domain::apply_filters::apply_filters;
use crate::domain::apply_format::apply_format;
use crate::domain::apply_search::apply_search;
use crate::models::{filter::Filter, format::Format, log_line::LogLine};
use crate::stores::analysis_store::AnalysisStore;
use crate::stores::log_store::LogStore;
use crate::stores::processing_store::ProcessingStore;

use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Currently processing lines (from, to)
    Processing(usize, usize),
    // New lines processed (from, to)
    NewLines(usize, usize),
    // New search lines processed (from, to)
    NewSearchLines(usize, usize),
    // Currently busy filtering
    Filtering,
    // Finished filtering
    FilterFinished,
    // Finished busy searching
    Searching,
    // Finished search
    SearchFinished,
}

#[async_trait]
pub trait LogAnalyzer {
    /// Add a new log source to the analysis
    async fn add_log(
        &self,
        source_type: usize,
        source_address: &String,
        format: Option<&String>,
    ) -> Result<()>;
    /// Add a new format to the list of available formats
    fn add_format(&self, alias: &String, regex: &String) -> Result<()>;
    /// Start a new search
    fn add_search(&self, regex: &String);
    /// Add a new filter to the list of available filters
    fn add_filter(&self, filter: Filter);
    /// Get log lines between the range [from, to]
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    /// Get search lines between the range [from, to]
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    /// Get a list of log lines of `elements` size centered on the `line` element or the closest
    /// Returns (elements, offset, index)
    fn get_log_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);

    /// Get a list of log lines of `elements` size centered on the `line` element or the closest
    /// Returns (elements, offset, index)
    fn get_search_lines_containing(
        &self,
        line: LogLine,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);

    /// Get the current managed logs
    /// Returns a vector of (enabled, log_path, Option<format>)
    fn get_logs(&self) -> Vec<(bool, String, Option<String>)>;

    /// Get all the available formats
    fn get_formats(&self) -> Vec<Format>;
    /// Get all the available filters together with their enabled state
    fn get_filters(&self) -> Vec<(bool, Filter)>;
    /// Get how many lines are in the raw logs
    fn get_total_raw_lines(&self) -> usize;
    /// Get how many lines are in the filtered log
    fn get_total_filtered_lines(&self) -> usize;
    /// Get how many lines are in the search log
    fn get_total_searched_lines(&self) -> usize;
    /// Enable or disable the given filter
    async fn toggle_filter(&self, id: &String);
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
        let (broadcast_sender, _broadcast_receiver) = broadcast::channel(1_000_000_usize);

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
            format_regex = format.map(|format| Regex::new(&format).unwrap());
        }

        let mut log_lines: Vec<LogLine> = Vec::with_capacity(line_index.len());
        for (line, index) in line_index {
            let log_line = apply_format(&format_regex.as_ref(), line, *index);
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
                    if apply_search(&search_regex, line) {
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
        log_store.add_log(source_address, log_source.clone(), format, true);

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
        let re = Regex::new(regex);
        self.analysis_store.reset_search();

        if re.is_ok() {
            self.analysis_store.add_search_query(regex);

            let analysis_store = self.analysis_store.clone();
            let regex_str = regex.clone();
            let sender = self.event_channel.clone();

            std::thread::Builder::new()
                .name("Search".to_string())
                .spawn(move || {
                    let r_lock = analysis_store.fetch_log();
                    let log = r_lock.read();

                    if !log.is_empty() {
                        sender.send(Event::Searching).unwrap_or_default();
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
                        })
                        .unwrap();
                        sender.send(Event::SearchFinished).unwrap_or_default();
                    }
                })
                .unwrap();
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

    async fn toggle_filter(&self, id: &String) {
        self.processing_store.toggle_filter(id);
        self.analysis_store.reset_log();
        self.analysis_store.reset_search();

        let mut receiver = self.event_channel.subscribe();

        let enabled_logs: Vec<String> = self
            .log_store
            .get_logs()
            .into_iter()
            .filter(|(enabled, _, _)| *enabled)
            .map(|(_, id, _)| id)
            .collect();

        let log_store = self.log_store.clone();
        let sender = self.log_sender.clone();
        let event_sender = self.event_channel.clone();

        std::thread::Builder::new()
            .name("Toggle filter".to_string())
            .spawn(move || {
                for log in enabled_logs {
                    let lines = log_store.extract_lines(&log);

                    if lines.is_empty() {
                        event_sender.send(Event::FilterFinished).unwrap();
                        continue;
                    }

                    event_sender.send(Event::Filtering).unwrap();
                    sender.send((log.clone(), lines.to_vec())).unwrap();

                    while !matches!(
                        async_std::task::block_on(receiver.recv()).unwrap_or(Event::Filtering),
                        Event::NewLines(_, last) if last == (lines.len() - 1)
                    ) {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    event_sender.send(Event::FilterFinished).unwrap();
                }
            })
            .unwrap();
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
