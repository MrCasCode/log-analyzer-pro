use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use flume::Sender;
use log_source::source::log_source::{create_source, LogSource, SourceType};
use regex::Regex;
use tokio::sync::broadcast;

use pariter::{scope, IteratorExt as _};

use crate::domain::apply_filters::apply_filters;
use crate::domain::apply_format::apply_format;
use crate::domain::apply_search::{apply_search, format_search};
use crate::models::filter::LogFilter;
use crate::models::log_line_styled::LogLineStyled;
use crate::models::{filter::Filter, format::Format, log_line::LogLine};
use crate::stores::analysis_store::AnalysisStore;
use crate::stores::log_store::LogStore;
use crate::stores::processing_store::ProcessingStore;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Notify of state changes
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

/// Main API of this crate
pub trait LogAnalyzer {
    /// Add a new log source to the analysis
    fn add_log(
        &self,
        source_type: usize,
        source_address: &str,
        format: Option<&String>,
    ) -> Result<()>;
    /// Add a new format to the list of available formats
    fn add_format(&self, alias: &str, regex: &str) -> Result<()>;
    /// Start a new search
    fn add_search(&self, regex: &str);
    /// Add a new filter to the list of available filters
    fn add_filter(&self, filter: Filter);
    /// Get log lines between the range [from, to]
    fn get_log_lines(&self, from: usize, to: usize) -> Vec<LogLine>;
    /// Get search lines between the range [from, to]
    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLineStyled>;
    /// Get a list of log lines of `elements` size centered on the `line` element or the closest
    /// Returns (elements, offset, index)
    fn get_log_lines_containing(
        &self,
        index: usize,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize);

    /// Get a list of log lines of `elements` size centered on the `line` element or the closest
    /// Returns (elements, offset, index)
    fn get_search_lines_containing(
        &self,
        index: usize,
        elements: usize,
    ) -> (Vec<LogLineStyled>, usize, usize);

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
    /// Enable or disable the given source
    fn toggle_source(&self, id: &str);
    /// Enable or disable the given filter
    fn toggle_filter(&self, id: &str);
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
    /// Instantiates the service and starts the consumer thread.
    ///
    /// The consumer thread continuously listens to lines from log sources and applies
    /// a chain of operations
    /// * apply format
    /// * apply filters
    /// * apply search
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
                    let (format, indexes, lines) = log.process_raw_lines(&path, lines);

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
                            // Split the lines to process in equal chunks to be processed in parallel
                            let processed: Vec<(Vec<LogLine>, Vec<LogLine>)> = elements
                                .chunks(chunk_size.max(num_cpus))
                                .parallel_map_scoped(scope, |chunk| {
                                    let lines = log.apply_format(&format, &path, chunk);
                                    let filtered_lines = log.apply_filters(lines);
                                    let (filtered, search) = log.apply_search(filtered_lines);
                                    (filtered, search)
                                })
                                .collect();

                            // Store the processed lines in the analysis store
                            for (filtered, search) in processed {
                                log.analysis_store.add_lines(&filtered);
                                log.analysis_store.add_search_lines(&search);
                            }

                            // Notify of the processed lines
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

    /// Store the raw received lines in memory and retrieve if there is a format for this log
    fn process_raw_lines(
        &self,
        path: &str,
        lines: Vec<String>,
    ) -> (Option<String>, Range<usize>, Vec<String>) {
        let indexes = self.log_store.add_lines(path, &lines);
        let format = self.log_store.get_format(path);
        (format, indexes, lines)
    }

    /// Apply formatting (if any) to a list of lines and return the formated `LogLine`
    fn apply_format(
        &self,
        format: &Option<String>,
        path: &str,
        line_index: &[(String, usize)],
    ) -> Vec<LogLine> {
        let mut format_regex = None;

        if let Some(format) = format {
            let format = self.processing_store.get_format(format);
            format_regex = format.map(|format| Regex::new(&format).unwrap());
        }

        let mut log_lines: Vec<LogLine> = Vec::with_capacity(line_index.len());
        for (line, index) in line_index {
            let log_line = apply_format(&format_regex.as_ref(), path, line, *index);
            log_lines.push(log_line);
        }
        log_lines
    }

    /// Apply filters (if any) to a list of `LogLine` and return the filtered list of `LogLine`
    fn apply_filters(&self, lines: Vec<LogLine>) -> Vec<LogLine> {
        let filters: Vec<LogFilter> = self
            .processing_store
            .get_filters()
            .into_iter()
            .filter(|(enabled, _)| *enabled)
            .map(|(_, filter)| filter.into())
            .collect();

        let mut filtered_lines: Vec<LogLine> = Vec::with_capacity(lines.len());
        for line in lines {
            if let Some(filtered_line) = apply_filters(&filters, line) {
                filtered_lines.push(filtered_line);
            }
        }
        filtered_lines
    }

    /// Apply the search query (if any) to a list of `LogLine` and return both the received lines and the searched ones
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

    /// Helper function to run log sources
    fn run_log_source(&self, log_source: Arc<Box<dyn LogSource + Send + Sync>>) {
        let sender = self.log_sender.clone();

        std::thread::Builder::new()
            .name(log_source.get_address())
            .spawn(|| {
                async_std::task::spawn(async move {
                    log_source.run(sender).await.unwrap();
                });
            })
            .unwrap();
    }
}

impl LogAnalyzer for LogService {
    fn add_log(
        &self,
        source_type: usize,
        source_address: &str,
        format: Option<&String>,
    ) -> Result<()> {
        let log_store = self.log_store.clone();

        let source_type = SourceType::try_from(source_type).unwrap();

        let log_source = Arc::new(async_std::task::block_on(create_source(
            source_type,
            source_address.to_string(),
        ))?);
        log_store.add_log(source_address, log_source.clone(), format, true);
        self.run_log_source(log_source);

        Ok(())
    }

    fn add_format(&self, alias: &str, regex: &str) -> Result<()> {
        let format = Format::new(alias, regex)?;

        self.processing_store.add_format(format.alias, format.regex);
        Ok(())
    }

    fn add_search(&self, regex: &str) {
        let re = Regex::new(regex);
        self.analysis_store.reset_search();

        if re.is_ok() {
            self.analysis_store.add_search_query(regex);

            let analysis_store = self.analysis_store.clone();
            let regex_str = regex.to_string();
            let sender = self.event_channel.clone();

            std::thread::Builder::new()
                .name("Search".to_string())
                .spawn(move || {
                    let log = analysis_store.fetch_log();

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

    fn get_search_lines(&self, from: usize, to: usize) -> Vec<LogLineStyled> {
        let search_lines_containing = self.analysis_store.get_search_lines(from, to);
        let mut styled_search_lines = vec![];

        if !search_lines_containing.is_empty() {
            // If there are search lines we are sure that there is a valid search query
            let query = Regex::new(&self.analysis_store.get_search_query().unwrap()).unwrap();
            styled_search_lines = search_lines_containing
                .into_iter()
                .map(|l| format_search(&query, &l))
                .collect();
        }

        styled_search_lines
    }

    fn get_log_lines_containing(
        &self,
        index: usize,
        elements: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        self.analysis_store
            .get_log_lines_containing(index, elements)
    }

    fn get_search_lines_containing(
        &self,
        index: usize,
        elements: usize,
    ) -> (Vec<LogLineStyled>, usize, usize) {
        let search_lines_containing = self
            .analysis_store
            .get_search_lines_containing(index, elements);

        let mut styled_search_lines: (Vec<LogLineStyled>, usize, usize) =
            (vec![], search_lines_containing.1, search_lines_containing.2);

        if !search_lines_containing.0.is_empty() {
            // If there are search lines we are sure that there is a valid search query
            let query = Regex::new(&self.analysis_store.get_search_query().unwrap()).unwrap();
            styled_search_lines.0 = search_lines_containing
                .0
                .into_iter()
                .map(|l| format_search(&query, &l))
                .collect();
        }

        styled_search_lines
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

    fn get_total_raw_lines(&self) -> usize {
        self.log_store.get_total_lines()
    }

    fn get_total_filtered_lines(&self) -> usize {
        self.analysis_store.get_total_filtered_lines()
    }

    fn get_total_searched_lines(&self) -> usize {
        self.analysis_store.get_total_searched_lines()
    }

    fn toggle_source(&self, id: &str) {
        if let Some((enabled, _log, _format)) = self
            .log_store
            .get_logs()
            .into_iter()
            .find(|(_, log_id, _)| log_id == id)
        {
            if let Some(source) = self.log_store.get_source(id) {
                self.log_store.toggle_log(id);
                // If enabled -> disable
                if enabled {
                    source.stop();
                } else {
                    self.run_log_source(source);
                }
            }
        }
    }

    fn toggle_filter(&self, id: &str) {
        self.processing_store.toggle_filter(id);

        // Reset everything because we need to recompute the log from the raw lines
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

    fn on_event(&self) -> broadcast::Receiver<Event> {
        self.event_channel.subscribe()
    }
}
