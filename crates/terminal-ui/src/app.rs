use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use log_analyzer::models::filter::FilterAction;
use log_analyzer::models::{filter::Filter, log_line::LogLine};
use log_analyzer::services::log_service::LogAnalyzer;

use std::sync::{Arc, RwLock};

use tui_input::backend::crossterm as input_backend;
use tui_input::Input;

use crate::data::lazy_stateful_table::{LazySource, LazyStatefulTable};
use crate::data::stateful_list::StatefulList;
use crate::data::stateful_table::StatefulTable;
use crate::data::Stateful;

/* ------ NEW SOURCE INDEXES ------- */
pub const INDEX_SOURCE_TYPE: usize = 0;
pub const INDEX_SOURCE_PATH: usize = INDEX_SOURCE_TYPE + 1;
pub const INDEX_SOURCE_FORMAT: usize = INDEX_SOURCE_PATH + 1;
pub const INDEX_SOURCE_NEW_FORMAT_ALIAS: usize = INDEX_SOURCE_FORMAT + 1;
pub const INDEX_SOURCE_NEW_FORMAT_REGEX: usize = INDEX_SOURCE_NEW_FORMAT_ALIAS + 1;
pub const INDEX_SOURCE_OK_BUTTON: usize = INDEX_SOURCE_NEW_FORMAT_REGEX + 1;
/* ------ FILTER INDEXES ------- */
pub const INDEX_FILTER_NAME: usize = INDEX_SOURCE_OK_BUTTON + 1;
pub const INDEX_FILTER_TYPE: usize = INDEX_FILTER_NAME + 1;
pub const INDEX_FILTER_DATETIME: usize = INDEX_FILTER_TYPE + 1;
pub const INDEX_FILTER_TIMESTAMP: usize = INDEX_FILTER_DATETIME + 1;
pub const INDEX_FILTER_APP: usize = INDEX_FILTER_TIMESTAMP + 1;
pub const INDEX_FILTER_SEVERITY: usize = INDEX_FILTER_APP + 1;
pub const INDEX_FILTER_FUNCTION: usize = INDEX_FILTER_SEVERITY + 1;
pub const INDEX_FILTER_PAYLOAD: usize = INDEX_FILTER_FUNCTION + 1;
pub const INDEX_FILTER_RED_COLOR: usize = INDEX_FILTER_PAYLOAD + 1;
pub const INDEX_FILTER_GREEN_COLOR: usize = INDEX_FILTER_RED_COLOR + 1;
pub const INDEX_FILTER_BLUE_COLOR: usize = INDEX_FILTER_GREEN_COLOR + 1;
pub const INDEX_FILTER_OK_BUTTON: usize = INDEX_FILTER_BLUE_COLOR + 1;
/* ------ SEARCH INDEXES ------- */
pub const INDEX_SEARCH: usize = INDEX_FILTER_OK_BUTTON + 1;
/* ----------------------------------- */
pub const INDEX_MAX: usize = INDEX_SEARCH + 1;
/* ----------------------------------- */

pub struct PopupInteraction {
    pub response: bool,
    pub message: String,
    pub calling_module: Module,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Module {
    Sources,
    Filters,
    Logs,
    Search,
    SearchResult,
    SourcePopup,
    FilterPopup,
    ErrorPopup,
    None,
}

struct LogSourcer {
    log_analyzer: Box<Arc<dyn LogAnalyzer>>,
}

impl LazySource<LogLine> for LogSourcer {
    fn source(&self, from: usize, to: usize) -> Vec<LogLine> {
        self.log_analyzer.get_log_lines(from, to)
    }
}
struct SearchSourcer {
    log_analyzer: Box<Arc<dyn LogAnalyzer>>,
}

impl LazySource<LogLine> for SearchSourcer {
    fn source(&self, from: usize, to: usize) -> Vec<LogLine> {
        self.log_analyzer.get_search_lines(from, to)
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub struct App {
    pub log_analyzer: Box<Arc<dyn LogAnalyzer>>,

    pub selected_module: Module,

    pub show_source_popup: bool,
    pub show_filter_popup: bool,

    pub input_buffers: Vec<Input>,
    pub input_buffer_index: usize,
    pub formats: StatefulList<String>,

    /// Tab selector index for Source Type
    pub source_type: usize,
    /// Tab selector index for Filter Type
    pub filter_type: usize,
    /// Tab selector index for Filter Type
    pub filter_color: usize,

    // Display all log sources in the sources panel
    pub sources: StatefulTable<(bool, String, Option<String>)>,
    // Display all filters in the filters panel
    pub filters: StatefulTable<(bool, String)>,

    pub log_lines: LazyStatefulTable<LogLine>,
    pub search_lines: LazyStatefulTable<LogLine>,
    pub horizontal_offset: usize,

    pub side_main_size_percentage: u16,
    pub log_filter_size_percentage: u16,
    pub log_search_size_percentage: u16,

    pub log_columns: Vec<(String, bool)>,

    pub show_error_message: bool,

    pub popup: PopupInteraction,
}

impl App {
    pub async fn new(log_analyzer: Box<Arc<dyn LogAnalyzer>>) -> App {
        let mut formats = vec!["New".to_string()];
        formats.extend(
            log_analyzer
                .get_formats()
                .into_iter()
                .map(|format| format.alias),
        );

        let sources = Arc::new(RwLock::new(log_analyzer.get_logs()));
        let filters = Arc::new(RwLock::new(
            log_analyzer
                .get_filters()
                .iter()
                .map(|(enabled, filter)| (*enabled, filter.alias.clone()))
                .collect(),
        ));

        let log_sourcer = LogSourcer {
            log_analyzer: log_analyzer.clone(),
        };
        let search_sourcer = SearchSourcer {
            log_analyzer: log_analyzer.clone(),
        };

        App {
            log_analyzer,
            selected_module: Module::Sources,
            show_source_popup: false,
            show_filter_popup: false,

            input_buffers: vec![Input::default(); INDEX_MAX],
            input_buffer_index: 0,

            formats: StatefulList::with_items(formats),

            source_type: 0,
            filter_type: 0,
            filter_color: 0,

            sources: StatefulTable::with_items(sources),
            filters: StatefulTable::with_items(filters),

            log_lines: LazyStatefulTable::new(Box::new(log_sourcer)),
            search_lines: LazyStatefulTable::new(Box::new(search_sourcer)),
            horizontal_offset: 0,
            log_filter_size_percentage: 50,
            log_search_size_percentage: 75,
            side_main_size_percentage: 25,
            log_columns: LogLine::columns()
                .into_iter()
                .map(|column| (column, true))
                .collect(),

            show_error_message: false,
            popup: PopupInteraction {
                response: true,
                calling_module: Module::None,
                message: String::new(),
            },
        }
    }

    pub async fn add_log(&mut self) -> Result<()> {
        let selected_format_index = self.formats.state.selected().unwrap(); // There is always one item selected

        let alias = match selected_format_index {
            0 /* NEW */ => {
                let alias = self.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS]
                    .value()
                    .to_string();
                let regex = self.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX]
                    .value()
                    .to_string();

                if !alias.is_empty() {
                    self.log_analyzer.add_format(&alias, &regex)?;
                    self.update_formats().await;
                    Some(alias)
                } else {
                    None
                }

            },
            _ => Some(self.formats.items[selected_format_index].clone())
        };

        let path = self.input_buffers[INDEX_SOURCE_PATH].value().to_string();
        self.log_analyzer
            .add_log(self.source_type, &path, alias.as_ref())
            .await?;

        Ok(())
    }

    pub async fn update_formats(&mut self) {
        let mut formats = vec!["New".to_string()];
        formats.extend(
            self.log_analyzer
                .get_formats()
                .into_iter()
                .map(|format| format.alias),
        );

        self.formats = StatefulList::with_items(formats);
        self.formats.state.select(Some(0));
    }

    pub async fn update_sources(&mut self) {
        let sources = self.log_analyzer.get_logs();
        self.sources = StatefulTable::with_items(Arc::new(RwLock::new(sources)))
    }

    pub async fn update_filters(&mut self) {
        let filters: Vec<(bool, String)> = self
            .log_analyzer
            .get_filters()
            .iter()
            .map(|(enabled, filter)| (*enabled, filter.alias.clone()))
            .collect();

        let index = self.filters.state.selected();
        let length: usize = filters.len();
        self.filters = StatefulTable::with_items(Arc::new(RwLock::new(filters)));

        if index.is_some() && length >= index.unwrap() {
            self.filters.state.select(index)
        }
    }

    async fn on_event(&mut self) {}

    pub async fn on_tick(&mut self) {
        self.on_event().await;
    }

    pub async fn handle_input(&mut self, key: KeyEvent) {
        match self.selected_module {
            Module::Sources => self.handle_sources_input(key).await,
            Module::Filters => self.handle_filters_input(key).await,
            Module::Logs => self.handle_log_input(key).await,
            Module::Search => self.handle_search_input(key).await,
            Module::SearchResult => self.handle_search_result_input(key).await,
            Module::SourcePopup => self.handle_source_popup_input(key).await,
            Module::FilterPopup => self.handle_filter_popup_input(key).await,
            Module::ErrorPopup => self.handle_error_popup_input(key).await,
            _ => {}
        }
    }

    async fn handle_sources_input(&mut self, key: KeyEvent) {
        if key.modifiers == KeyModifiers::SHIFT {
            match key.code {
                KeyCode::Char('W') => {
                    App::decrease_ratio(&mut self.log_filter_size_percentage, 5, 20)
                }
                KeyCode::Char('S') => {
                    App::increase_ratio(&mut self.log_filter_size_percentage, 5, 80)
                }
                KeyCode::Char('A') => {
                    App::decrease_ratio(&mut self.side_main_size_percentage, 5, 0)
                }
                KeyCode::Char('D') => {
                    App::increase_ratio(&mut self.side_main_size_percentage, 5, 50)
                }
                _ => {}
            };
        }

        match key.code {
            // Navigate up sources
            KeyCode::Up => {
                self.sources.previous();
            }
            // Navigate down sources
            KeyCode::Down => {
                self.sources.next();
            }
            // Toggle enabled/disabled source
            KeyCode::Enter => {}
            // Add new source -> Popup window
            KeyCode::Char('i') | KeyCode::Char('+') | KeyCode::Char('a') => {
                self.formats.state.select(Some(0));
                self.show_source_popup = true;
                self.input_buffer_index = INDEX_SOURCE_TYPE;
                self.selected_module = Module::SourcePopup;
            }
            // Delete source
            KeyCode::Char('-') | KeyCode::Char('d') | KeyCode::Delete | KeyCode::Backspace => {}
            // Nothing
            _ => {}
        }
    }

    async fn handle_filters_input(&mut self, key: KeyEvent) {
        if key.modifiers == KeyModifiers::SHIFT {
            match key.code {
                KeyCode::Char('W') => {
                    App::decrease_ratio(&mut self.log_filter_size_percentage, 5, 20)
                }
                KeyCode::Char('S') => {
                    App::increase_ratio(&mut self.log_filter_size_percentage, 5, 80)
                }
                KeyCode::Char('A') => {
                    App::decrease_ratio(&mut self.side_main_size_percentage, 5, 0)
                }
                KeyCode::Char('D') => {
                    App::increase_ratio(&mut self.side_main_size_percentage, 5, 50)
                }
                _ => {}
            };
        }
        match key.code {
            // Navigate up filters
            KeyCode::Up => {
                self.filters.previous();
            }
            // Navigate down filters
            KeyCode::Down => {
                self.filters.next();
            }
            // Toggle enabled/disabled source
            KeyCode::Enter => {
                if let Some(index) = self.filters.state.selected() {
                    let (_, alias) = &self.filters.items.read().unwrap()[index];
                    self.log_analyzer.toggle_filter(alias);
                }
                self.update_filters().await;
            }
            // Add new filter -> Popup window
            KeyCode::Char('i') | KeyCode::Char('+') | KeyCode::Char('a') => {
                self.show_filter_popup = true;
                self.input_buffer_index = INDEX_FILTER_NAME;
                self.selected_module = Module::FilterPopup;
            }
            // Delete source
            KeyCode::Char('-') | KeyCode::Char('d') | KeyCode::Delete => {}
            // Nothing
            _ => {}
        }
    }

    async fn handle_log_input(&mut self, key: KeyEvent) {
        self.handle_table_input(Module::Logs, key).await;
    }

    async fn handle_search_result_input(&mut self, key: KeyEvent) {
        self.handle_table_input(Module::SearchResult, key).await;
    }

    async fn handle_search_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.log_analyzer
                    .add_search(&self.input_buffers[INDEX_SEARCH].value().into());
            }
            _ => {
                input_backend::to_input_request(Event::Key(key))
                    .map(|req| self.input_buffers[INDEX_SEARCH].handle(req));
            }
        }
    }

    async fn handle_source_popup_input(&mut self, key: KeyEvent) {
        let mut fill_format = |_: usize, current_format: &str| match current_format {
            "New" => {
                self.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS] = Input::default();
                self.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX] = Input::default();
            }
            alias => {
                let format = self
                    .log_analyzer
                    .get_formats()
                    .iter()
                    .find(|format| format.alias == alias)
                    .unwrap()
                    .clone();
                self.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS] =
                    Input::default().with_value(format.alias);
                self.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX] =
                    Input::default().with_value(format.regex);
            }
        };
        // Add new source -> Popup window
        if key.code == KeyCode::Esc {
            self.show_source_popup = false;
            self.selected_module = Module::Sources;
            return;
        }

        match self.input_buffer_index {
            INDEX_SOURCE_TYPE => {
                // Switch between file and ws
                if key.code == KeyCode::Right || key.code == KeyCode::Left {
                    self.source_type = !self.source_type & 1;
                }
            }
            INDEX_SOURCE_FORMAT => match key.code {
                // Navigate up sources
                KeyCode::Up => {
                    if self.input_buffer_index == INDEX_SOURCE_FORMAT {
                        let i = self.formats.previous();
                        fill_format(i, self.formats.items[i].as_str());
                    }
                }
                // Navigate down sources
                KeyCode::Down => {
                    if self.input_buffer_index == INDEX_SOURCE_FORMAT {
                        let i = self.formats.next();
                        fill_format(i, self.formats.items[i].as_str());
                    }
                }
                _ => {}
            },
            index @ (INDEX_SOURCE_PATH
            | INDEX_SOURCE_NEW_FORMAT_ALIAS
            | INDEX_SOURCE_NEW_FORMAT_REGEX) => {
                input_backend::to_input_request(Event::Key(key))
                    .map(|req| self.input_buffers[index].handle(req));
            }
            INDEX_SOURCE_OK_BUTTON => {
                if key.code == KeyCode::Enter {
                    match self.add_log().await {
                        Ok(_) => {
                            self.show_source_popup = false;
                            self.selected_module = Module::Sources;
                            self.update_sources().await;
                        }
                        Err(err) => {
                            self.selected_module = Module::ErrorPopup;
                            self.show_error_message = true;
                            self.popup.message = format!("{:?}", err);
                            self.popup.calling_module = Module::SourcePopup;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_filter_popup_input(&mut self, key: KeyEvent) {
        // Add new filter -> Popup window
        if key.code == KeyCode::Esc {
            self.show_filter_popup = false;
            self.selected_module = Module::Filters;
            return;
        }

        match self.input_buffer_index {
            index @ (INDEX_FILTER_NAME
            | INDEX_FILTER_DATETIME
            | INDEX_FILTER_TIMESTAMP
            | INDEX_FILTER_APP
            | INDEX_FILTER_SEVERITY
            | INDEX_FILTER_FUNCTION
            | INDEX_FILTER_PAYLOAD
            | INDEX_FILTER_RED_COLOR
            | INDEX_FILTER_GREEN_COLOR
            | INDEX_FILTER_BLUE_COLOR) => {
                input_backend::to_input_request(Event::Key(key))
                    .map(|req| self.input_buffers[index].handle(req));
            }
            INDEX_FILTER_TYPE => {
                // Switch tabs
                if key.code == KeyCode::Right || key.code == KeyCode::Left {
                    let circular_choice = |i: &mut usize, max, add: i32| {
                        *i = match (*i as i32 + add) as i32 {
                            r if r > max => 0_usize,    // if adding overflows -> set to 0
                            r if r < 0 => max as usize, // if adding underflows -> set to 0
                            r => r as usize,
                        }
                    };

                    let sum = if key.code == KeyCode::Right { 1 } else { -1 };
                    if self.input_buffer_index == INDEX_FILTER_TYPE {
                        circular_choice(&mut self.filter_type, 2, sum)
                    }
                }
            }

            INDEX_FILTER_OK_BUTTON => {
                if key.code == KeyCode::Enter {
                    let filter = Filter {
                        alias: self.input_buffers[INDEX_FILTER_NAME].value().to_string(),
                        action: FilterAction::from(self.filter_type),
                        filter: LogLine {
                            index: "".to_string(),
                            date: self.input_buffers[INDEX_FILTER_DATETIME]
                                .value()
                                .to_string(),
                            timestamp: self.input_buffers[INDEX_FILTER_TIMESTAMP]
                                .value()
                                .to_string(),
                            app: self.input_buffers[INDEX_FILTER_APP].value().to_string(),
                            severity: self.input_buffers[INDEX_FILTER_SEVERITY]
                                .value()
                                .to_string(),
                            function: self.input_buffers[INDEX_FILTER_FUNCTION]
                                .value()
                                .to_string(),
                            payload: self.input_buffers[INDEX_FILTER_PAYLOAD].value().to_string(),
                            color: parse_color(
                                self.input_buffers[INDEX_FILTER_RED_COLOR].value(),
                                self.input_buffers[INDEX_FILTER_GREEN_COLOR].value(),
                                self.input_buffers[INDEX_FILTER_BLUE_COLOR].value(),
                            ),
                        },
                    };
                    self.log_analyzer.add_filter(filter);
                    self.show_filter_popup = false;
                    self.selected_module = Module::Filters;
                    self.update_filters().await;
                }
            }
            _ => {}
        }
    }

    async fn handle_error_popup_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.show_error_message = false;
                self.popup.response = true;
                self.selected_module = self.popup.calling_module;
            }
            _ => {}
        }
    }

    pub fn navigate(&mut self, direction: KeyCode) {
        match self.selected_module {
            Module::Sources => {
                match direction {
                    KeyCode::Up | KeyCode::Down => self.selected_module = Module::Filters,
                    KeyCode::Left | KeyCode::Right => self.selected_module = Module::Logs,
                    _ => {}
                };
                self.sources.unselect()
            }
            Module::Filters => {
                match direction {
                    KeyCode::Up | KeyCode::Down => self.selected_module = Module::Sources,
                    KeyCode::Left | KeyCode::Right => self.selected_module = Module::Search,
                    _ => {}
                };
                self.filters.unselect()
            }
            Module::Logs => match direction {
                KeyCode::Up => self.selected_module = Module::SearchResult,
                KeyCode::Down => self.selected_module = Module::Search,
                KeyCode::Left | KeyCode::Right => {
                    if self.side_main_size_percentage > 0 {
                        self.selected_module = Module::Sources
                    }
                }
                _ => {}
            },
            Module::Search => match direction {
                KeyCode::Up => self.selected_module = Module::Logs,
                KeyCode::Down => self.selected_module = Module::SearchResult,
                KeyCode::Left | KeyCode::Right => {
                    if self.side_main_size_percentage > 0 {
                        self.selected_module = Module::Filters
                    }
                }
                _ => {}
            },
            Module::SearchResult => match direction {
                KeyCode::Up => self.selected_module = Module::Search,
                KeyCode::Down => self.selected_module = Module::Logs,
                KeyCode::Left | KeyCode::Right => {
                    if self.side_main_size_percentage > 0 {
                        self.selected_module = Module::Filters
                    }
                }
                _ => {}
            },
            Module::SourcePopup => {
                match direction {
                    // Navigate up sources
                    KeyCode::Up => {
                        if self.input_buffer_index > INDEX_SOURCE_TYPE {
                            self.input_buffer_index -= 1;
                        }
                    }
                    // Navigate down sources
                    KeyCode::Down => {
                        if self.input_buffer_index < INDEX_SOURCE_OK_BUTTON {
                            self.input_buffer_index += 1;
                        }
                    }
                    _ => {}
                }
            }
            Module::FilterPopup => {
                match direction {
                    // Navigate up sources
                    KeyCode::Up => {
                        if self.input_buffer_index > INDEX_FILTER_NAME {
                            self.input_buffer_index -= 1;
                        }
                    }
                    // Navigate down sources
                    KeyCode::Down => {
                        if self.input_buffer_index < INDEX_FILTER_OK_BUTTON {
                            self.input_buffer_index += 1;
                        }
                    }
                    _ => {}
                }
            }
            Module::ErrorPopup => (),
            Module::None => self.selected_module = Module::Logs,
        }
    }

    fn increase_ratio(ratio: &mut u16, step: u16, max: u16) {
        *ratio = (*ratio + step).min(max)
    }

    fn decrease_ratio(ratio: &mut u16, step: u16, min: u16) {
        *ratio = if *ratio > min { *ratio - step } else { *ratio }
    }

    async fn handle_table_input(&mut self, module: Module, key: KeyEvent) {
        let table = if module == Module::Logs {
            &mut self.log_lines
        } else {
            &mut self.search_lines
        };
        let multiplier = if key.modifiers == KeyModifiers::ALT {
            10
        } else {
            1
        };
        match key.modifiers {
            KeyModifiers::SHIFT => match key.code {
                KeyCode::Char('W') => {
                    App::decrease_ratio(&mut self.log_search_size_percentage, 5, 10)
                }
                KeyCode::Char('S') => {
                    App::increase_ratio(&mut self.log_search_size_percentage, 5, 90)
                }
                KeyCode::Char('A') => {
                    App::decrease_ratio(&mut self.side_main_size_percentage, 5, 0)
                }
                KeyCode::Char('D') => {
                    App::increase_ratio(&mut self.side_main_size_percentage, 5, 50)
                }
                KeyCode::Char('G') => {}
                _ => {}
            },
            _ => match key.code {
                // Navigate up log_lines
                KeyCode::Up => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        table.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::Down => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        table.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::PageUp => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        table.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::PageDown => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        table.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::Left => {
                    self.horizontal_offset -= if self.horizontal_offset == 0 { 0 } else { 10 }
                }
                // Navigate down log_lines
                KeyCode::Right => self.horizontal_offset += 10,
                KeyCode::Char('I') | KeyCode::Char('i') => {
                    self.log_columns[0].1 = !self.log_columns[0].1
                }
                KeyCode::Char('D') | KeyCode::Char('d') => {
                    self.log_columns[1].1 = !self.log_columns[1].1
                }
                KeyCode::Char('T') | KeyCode::Char('t') => {
                    self.log_columns[2].1 = !self.log_columns[2].1
                }
                KeyCode::Char('A') | KeyCode::Char('a') => {
                    self.log_columns[3].1 = !self.log_columns[3].1
                }
                KeyCode::Char('S') | KeyCode::Char('s') => {
                    self.log_columns[4].1 = !self.log_columns[4].1
                }
                KeyCode::Char('F') | KeyCode::Char('f') => {
                    self.log_columns[5].1 = !self.log_columns[5].1
                }
                KeyCode::Char('P') | KeyCode::Char('p') => {
                    self.log_columns[6].1 = !self.log_columns[6].1
                }
                KeyCode::Enter => {
                    if module == Module::SearchResult {
                        let current_line =
                            &self.search_lines.items[self.search_lines.state.selected().unwrap()];
                        let index = current_line.index.clone();
                    }
                }
                // Nothing
                _ => {}
            },
        }
    }
}

pub fn parse_color(r: &str, g: &str, b: &str) -> Option<(u8, u8, u8)> {
    match (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>()) {
        parse
            if [&parse.0, &parse.1, &parse.2]
                .into_iter()
                .any(|p| p.is_ok()) =>
        {
            Some((
                parse.0.unwrap_or_default(),
                parse.1.unwrap_or_default(),
                parse.2.unwrap_or_default(),
            ))
        }
        _ => None,
    }
}
