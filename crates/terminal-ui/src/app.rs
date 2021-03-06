use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use log_analyzer::models::filter::FilterAction;
use log_analyzer::models::log_line_styled::LogLineStyled;
use log_analyzer::models::{filter::Filter, log_line::LogLine};
use log_analyzer::services::log_service::{Event as LogEvent, LogAnalyzer};
use tui::style::Color;

use std::sync::Arc;

use tui_input::backend::crossterm as input_backend;
use tui_input::Input;

use crate::data::lazy_stateful_table::{LazySource, LazyStatefulTable, CAPACITY};
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
pub const INDEX_FILTER_LOG: usize = INDEX_FILTER_TYPE + 1;
pub const INDEX_FILTER_DATETIME: usize = INDEX_FILTER_LOG + 1;
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
/* ------ NAVIGATION INDEXES ------- */
pub const INDEX_NAVIGATION: usize = INDEX_SEARCH + 1;
/* ----------------------------------- */
pub const INDEX_MAX: usize = INDEX_NAVIGATION + 1;
/* ----------------------------------- */

pub struct PopupInteraction {
    pub response: bool,
    pub message: String,
    pub calling_module: Module,
}

pub struct Processing {
    pub is_processing: bool,
    pub focus_on: usize,
}

impl Processing {
    fn set_focus(&mut self, focus: Option<usize>) {
        self.focus_on = match focus {
            Some(focus) => focus,
            None => 0,
        }
    }
}

impl Default for Processing {
    fn default() -> Self {
        Self {
            is_processing: false,
            focus_on: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Module {
    Sources,
    Filters,
    Logs,
    Search,
    SearchResult,
    SourcePopup,
    FilterPopup,
    NavigationPopup,
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

    fn source_elements_containing(
        &self,
        index: usize,
        quantity: usize,
    ) -> (Vec<LogLine>, usize, usize) {
        self.log_analyzer.get_log_lines_containing(index, quantity)
    }
}
struct SearchSourcer {
    log_analyzer: Box<Arc<dyn LogAnalyzer>>,
}

impl LazySource<LogLineStyled> for SearchSourcer {
    fn source(&self, from: usize, to: usize) -> Vec<LogLineStyled> {
        self.log_analyzer.get_search_lines(from, to)
    }

    fn source_elements_containing(
        &self,
        index: usize,
        quantity: usize,
    ) -> (Vec<LogLineStyled>, usize, usize) {
        self.log_analyzer
            .get_search_lines_containing(index, quantity)
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
pub struct App {
    /// Api to the backend processor
    pub log_analyzer: Box<Arc<dyn LogAnalyzer>>,

    /// Primary color
    pub color: Color,

    /// Currently selected module. Used to manage inputs and highlight focus
    pub selected_module: Module,

    /// Display the new source popup
    pub show_source_popup: bool,
    /// Display the new filter popup
    pub show_filter_popup: bool,
    /// Display an error message
    pub show_error_message: bool,
    /// Display the navigation popup
    pub show_navigation_popup: bool,
    /// Display the navigation popup
    pub show_log_options_popup: bool,

    /// Vector of user input. Entries are uniquely assigned to each UI input, and the selection is
    /// performed with the `input_buffer_index`
    pub input_buffers: Vec<Input>,
    /// Currently selected input buffer
    pub input_buffer_index: usize,
    /// Stateful list of all the current formats to be displayed in the source popup
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

    /// Lazy widget for the main view of the logs
    pub log_lines: LazyStatefulTable<LogLine>,
    /// Lazy widget for the main view of the search
    pub search_lines: LazyStatefulTable<LogLineStyled>,
    /// Apply an offset to the logs to simulate horizontal scrolling
    pub horizontal_offset: usize,

    /// Resizing of the side_menu to the main view
    pub side_main_size_percentage: u16,
    /// Resizing on the side_menu between sources and filters
    pub log_filter_size_percentage: u16,
    /// Resizing on the main view between logs and searchs
    pub log_search_size_percentage: u16,

    /// Active log columns to display in the log and the search
    pub log_columns: Vec<(String, bool)>,

    /// Auto scroll to the last receive elements. Used for live logs
    pub auto_scroll: bool,

    /// Manage the popup interaction
    pub popup: PopupInteraction,
    /// Manage the processing popup
    pub processing: Processing,
    /// Receive state events from the backed to kwow when it's busy or when new elements are available
    event_receiver: tokio::sync::broadcast::Receiver<LogEvent>,
}

impl App {
    pub async fn new(log_analyzer: Box<Arc<dyn LogAnalyzer>>, primary_color: Color) -> App {
        let mut formats = vec!["New".to_string()];
        formats.extend(
            log_analyzer
                .get_formats()
                .into_iter()
                .map(|format| format.alias),
        );

        let sources = log_analyzer.get_logs();
        let filters = log_analyzer
            .get_filters()
            .iter()
            .map(|(enabled, filter)| (*enabled, filter.alias.clone()))
            .collect();

        let log_sourcer = LogSourcer {
            log_analyzer: log_analyzer.clone(),
        };
        let search_sourcer = SearchSourcer {
            log_analyzer: log_analyzer.clone(),
        };

        let event_receiver = log_analyzer.on_event();

        App {
            log_analyzer,
            color: primary_color,
            selected_module: Module::Sources,
            show_source_popup: false,
            show_filter_popup: false,
            show_navigation_popup: false,
            show_error_message: false,
            show_log_options_popup: false,

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
            auto_scroll: false,

            popup: PopupInteraction {
                response: true,
                calling_module: Module::None,
                message: String::new(),
            },
            processing: Processing::default(),
            event_receiver,
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
            .add_log(self.source_type, &path, alias.as_ref())?;

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
        let index = self.sources.state.selected();
        let sources = self.log_analyzer.get_logs();
        self.sources = StatefulTable::with_items(sources);

        if index.is_some() && self.sources.items.len() >= index.unwrap() {
            self.sources.state.select(index)
        }
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
        self.filters = StatefulTable::with_items(filters);

        if index.is_some() && length >= index.unwrap() {
            self.filters.state.select(index)
        }
    }

    async fn pull_events(&mut self) {
        let mut events = Vec::new();
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }

        // Reload logs when some lines are received and there are no items displayed
        if !self.processing.is_processing
            && self.log_lines.items.len() < CAPACITY
            && events.iter().any(|e| matches!(e, LogEvent::NewLines(_, _)))
        {
            self.log_lines.reload();
        }

        // Reload search logs when some search lines are received and there are no items displayed
        if !self.processing.is_processing
            && self.search_lines.items.len() < CAPACITY
            && events
                .iter()
                .any(|e| matches!(e, LogEvent::NewSearchLines(_, _)))
        {
            self.search_lines.reload();
        }

        // Auto scroll
        if self.auto_scroll && events.iter().any(|e| matches!(e, LogEvent::NewLines(_, _))) {
            self.log_lines.navigate_to_bottom();
        }

        if self.auto_scroll
            && events
                .iter()
                .any(|e| matches!(e, LogEvent::NewSearchLines(_, _)))
        {
            self.search_lines.navigate_to_bottom();
        }

        // Handle enter filtering
        if events.iter().any(|e| matches!(e, LogEvent::Filtering)) {
            self.processing.is_processing = true;

            self.processing.set_focus(
                self.log_lines
                    .get_selected_item()
                    .map(|l| l.index.parse().unwrap()),
            );
            self.log_lines.clear();
            self.search_lines.clear();
        }

        // Handle exit filtering
        if self.processing.is_processing
            && events.iter().any(|e| matches!(e, LogEvent::FilterFinished))
        {
            self.log_lines.navigate_to(self.processing.focus_on);
            self.search_lines.navigate_to(self.processing.focus_on);

            self.processing.is_processing = false;
            self.processing = Processing::default();
        }

        // Handle enter searching
        if events.iter().any(|e| matches!(e, LogEvent::Searching)) {
            self.processing.is_processing = true;
            self.processing.set_focus(
                self.search_lines
                    .get_selected_item()
                    .map(|l| l.unformat().index.parse().unwrap()),
            );
            self.search_lines.clear();
        }

        // Handle exit searching
        if events.iter().any(|e| matches!(e, LogEvent::SearchFinished)) {
            self.processing.is_processing = false;

            self.search_lines.navigate_to(self.processing.focus_on);
            self.processing = Processing::default();
        }
    }

    pub async fn on_tick(&mut self) {
        self.pull_events().await;
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
            Module::NavigationPopup => self.handle_navigation_popup_input(key).await,
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
            KeyCode::Enter => {
                if let Some(i) = self.sources.state.selected() {
                    let (_, id, _) = &self.sources.items[i];
                    self.log_analyzer.toggle_source(id);
                    self.update_sources().await;
                }
            }
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
                    let (_, alias) = &self.filters.items[index];
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
            // Edit filter -> Popup window
            KeyCode::Char('e') => {
                self.show_filter_popup = true;
                self.input_buffer_index = INDEX_FILTER_NAME;
                self.selected_module = Module::FilterPopup;

                if let Some(i) = self.filters.state.selected() {
                    let (_, alias) = &self.filters.items[i];
                    if let Some((_, filter)) = self
                        .log_analyzer
                        .get_filters()
                        .into_iter()
                        .find(|(_, filter)| filter.alias == *alias)
                    {
                        self.filter_type = filter.action.into();
                        self.input_buffers[INDEX_FILTER_NAME] =
                            Input::default().with_value(alias.clone());
                        self.input_buffers[INDEX_FILTER_TYPE] =
                            Input::default().with_value("".into());
                        self.input_buffers[INDEX_FILTER_LOG] =
                            Input::default().with_value(filter.filter.log);
                        self.input_buffers[INDEX_FILTER_DATETIME] =
                            Input::default().with_value(filter.filter.date);
                        self.input_buffers[INDEX_FILTER_TIMESTAMP] =
                            Input::default().with_value(filter.filter.timestamp);
                        self.input_buffers[INDEX_FILTER_APP] =
                            Input::default().with_value(filter.filter.app);
                        self.input_buffers[INDEX_FILTER_SEVERITY] =
                            Input::default().with_value(filter.filter.severity);
                        self.input_buffers[INDEX_FILTER_FUNCTION] =
                            Input::default().with_value(filter.filter.function);
                        self.input_buffers[INDEX_FILTER_PAYLOAD] =
                            Input::default().with_value(filter.filter.payload);
                        if let Some((r, g, b)) = filter.filter.color {
                            self.input_buffers[INDEX_FILTER_RED_COLOR] =
                                Input::default().with_value(r.to_string());
                            self.input_buffers[INDEX_FILTER_GREEN_COLOR] =
                                Input::default().with_value(g.to_string());
                            self.input_buffers[INDEX_FILTER_BLUE_COLOR] =
                                Input::default().with_value(b.to_string());
                        }
                    }
                }
            }
            // Delete filter
            KeyCode::Char('-') | KeyCode::Char('d') | KeyCode::Delete => {}
            // Nothing
            _ => {}
        }
    }

    async fn handle_log_input(&mut self, key: KeyEvent) {
        self.handle_table_log_input(key).await;
    }

    async fn handle_search_result_input(&mut self, key: KeyEvent) {
        self.handle_table_search_input(key).await;
    }

    async fn handle_search_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.search_lines.clear();
                self.log_analyzer
                    .add_search(self.input_buffers[INDEX_SEARCH].value());
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
            self.source_type = 0;
            self.selected_module = Module::Sources;
            self.formats.state.select(Some(0));
            self.input_buffers[INDEX_SOURCE_TYPE..INDEX_SOURCE_NEW_FORMAT_REGEX]
                .iter_mut()
                .for_each(|b| *b = Input::default().with_value("".into()));
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
                            self.source_type = 0;
                            self.selected_module = Module::Sources;
                            self.update_sources().await;
                            self.input_buffers[INDEX_SOURCE_TYPE..INDEX_SOURCE_NEW_FORMAT_REGEX]
                                .iter_mut()
                                .for_each(|b| *b = Input::default().with_value("".into()));
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
            self.filter_type = 0;
            self.input_buffers[INDEX_FILTER_NAME..INDEX_FILTER_BLUE_COLOR]
                .iter_mut()
                .for_each(|b| *b = Input::default().with_value("".into()));
            return;
        }

        match self.input_buffer_index {
            index @ (INDEX_FILTER_NAME
            | INDEX_FILTER_LOG
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
                            log: self.input_buffers[INDEX_FILTER_LOG].value().to_string(),
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
                            ..Default::default()
                        },
                    };
                    self.log_analyzer.add_filter(filter);
                    self.show_filter_popup = false;
                    self.selected_module = Module::Filters;
                    self.filter_type = 0;
                    self.update_filters().await;
                    self.input_buffers[INDEX_FILTER_NAME..INDEX_FILTER_BLUE_COLOR]
                        .iter_mut()
                        .for_each(|b| *b = Input::default().with_value("".into()));
                }
            }
            _ => {}
        }
    }

    async fn handle_navigation_popup_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                match self.input_buffers[INDEX_NAVIGATION]
                    .value()
                    .parse::<usize>()
                {
                    Ok(index) => {
                        self.show_navigation_popup = false;
                        self.selected_module = self.popup.calling_module;
                        self.input_buffers[INDEX_NAVIGATION] =
                            Input::default().with_value("".into());

                        match self.selected_module {
                            Module::Logs => {
                                self.log_lines.navigate_to(index);
                            }
                            Module::SearchResult => {
                                self.search_lines.navigate_to(index);
                            }
                            _ => {}
                        }
                    }
                    Err(err) => {
                        self.selected_module = Module::ErrorPopup;
                        self.show_error_message = true;
                        self.popup.message = err.to_string();
                    }
                }
            }
            KeyCode::Esc => {
                self.show_navigation_popup = false;
                self.selected_module = self.popup.calling_module;
                self.input_buffers[INDEX_NAVIGATION] = Input::default().with_value("".into());
            }
            _ => {
                input_backend::to_input_request(Event::Key(key))
                    .map(|req| self.input_buffers[INDEX_NAVIGATION].handle(req));
            }
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
            Module::NavigationPopup => (),
            Module::None => self.selected_module = Module::Logs,
        }
    }

    fn increase_ratio(ratio: &mut u16, step: u16, max: u16) {
        *ratio = (*ratio + step).min(max)
    }

    fn decrease_ratio(ratio: &mut u16, step: u16, min: u16) {
        *ratio = if *ratio > min { *ratio - step } else { *ratio }
    }

    pub fn get_column_lenght(&self, column: &str) -> u16 {
        let lenght = |log_lines: &Vec<LogLine>| {
            log_lines
                .iter()
                .map(|l| l.get(column).unwrap())
                .max_by_key(|l| l.len())
                .map(|l| l.len().clamp(0, u16::MAX as usize) as u16)
        };

        let max_log_lenght = lenght(&self.log_lines.items);
        let max_search_lenght = lenght(
            &self
                .search_lines
                .items
                .iter()
                .map(|line| line.unformat())
                .collect(),
        );

        match (max_log_lenght, max_search_lenght) {
            (Some(l), Some(s)) => l.max(s),
            (Some(l), None) => l,
            (None, Some(s)) => s,
            _ => 15,
        }
    }

    async fn handle_table_log_input(&mut self, key: KeyEvent) {
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
                KeyCode::Char('G') => {
                    self.input_buffer_index = INDEX_NAVIGATION;
                    self.show_navigation_popup = true;
                    self.popup.calling_module = Module::Logs;
                    self.selected_module = Module::NavigationPopup;
                }
                _ => {}
            },
            _ => match key.code {
                // Navigate up log_lines
                KeyCode::Up => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        self.log_lines.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::Down => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        self.log_lines.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::PageUp => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        self.log_lines.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::PageDown => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        self.log_lines.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::Left => {
                    if self.horizontal_offset > 0 {
                        self.horizontal_offset -= if self.horizontal_offset == 0 { 0 } else { 10 };
                        return;
                    }
                    for (i, (column, enabled)) in self.log_columns.iter().enumerate().rev() {
                        if !*enabled && self.get_column_lenght(column) != 0 {
                            self.log_columns[i].1 = true;
                            return;
                        }
                    }
                }
                // Navigate down log_lines
                KeyCode::Right => {
                    for (i, (column, enabled)) in self.log_columns.iter().enumerate() {
                        if i != (self.log_columns.len() - 1)
                            && *enabled
                            && self.get_column_lenght(column) != 0
                        {
                            self.log_columns[i].1 = false;
                            return;
                        }
                    }
                    self.horizontal_offset += 10
                }
                // Toogle columns
                KeyCode::Char('l') => self.log_columns[0].1 = !self.log_columns[0].1,
                KeyCode::Char('i') => self.log_columns[1].1 = !self.log_columns[1].1,
                KeyCode::Char('d') => self.log_columns[2].1 = !self.log_columns[2].1,
                KeyCode::Char('t') => self.log_columns[3].1 = !self.log_columns[3].1,
                KeyCode::Char('a') => self.log_columns[4].1 = !self.log_columns[4].1,
                KeyCode::Char('s') => self.log_columns[5].1 = !self.log_columns[5].1,
                KeyCode::Char('f') => self.log_columns[6].1 = !self.log_columns[6].1,
                KeyCode::Char('p') => self.log_columns[7].1 = !self.log_columns[7].1,
                KeyCode::Char('r') => self.auto_scroll = !self.auto_scroll,
                // Nothing
                _ => {}
            },
        }
    }

    async fn handle_table_search_input(&mut self, key: KeyEvent){
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
                KeyCode::Char('G') => {
                    self.input_buffer_index = INDEX_NAVIGATION;
                    self.show_navigation_popup = true;
                    self.popup.calling_module = Module::SearchResult;
                    self.selected_module = Module::NavigationPopup;
                }
                _ => {}
            },
            _ => match key.code {
                // Navigate up log_lines
                KeyCode::Up => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        self.search_lines.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::Down => {
                    let steps = multiplier;
                    for _ in 0..steps {
                        self.search_lines.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::PageUp => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        self.search_lines.previous();
                    }
                }
                // Navigate down log_lines
                KeyCode::PageDown => {
                    let steps = 100 * multiplier;
                    for _ in 0..steps {
                        self.search_lines.next();
                    }
                }
                // Navigate up log_lines
                KeyCode::Left => {
                    if self.horizontal_offset > 0 {
                        self.horizontal_offset -= if self.horizontal_offset == 0 { 0 } else { 10 };
                        return;
                    }
                    for (i, (column, enabled)) in self.log_columns.iter().enumerate().rev() {
                        if !*enabled && self.get_column_lenght(column) != 0 {
                            self.log_columns[i].1 = true;
                            return;
                        }
                    }
                }
                // Navigate down log_lines
                KeyCode::Right => {
                    for (i, (column, enabled)) in self.log_columns.iter().enumerate() {
                        if i != (self.log_columns.len() - 1)
                            && *enabled
                            && self.get_column_lenght(column) != 0
                        {
                            self.log_columns[i].1 = false;
                            return;
                        }
                    }
                    self.horizontal_offset += 10
                }
                // Toogle columns
                KeyCode::Char('l') => self.log_columns[0].1 = !self.log_columns[0].1,
                KeyCode::Char('i') => self.log_columns[1].1 = !self.log_columns[1].1,
                KeyCode::Char('d') => self.log_columns[2].1 = !self.log_columns[2].1,
                KeyCode::Char('t') => self.log_columns[3].1 = !self.log_columns[3].1,
                KeyCode::Char('a') => self.log_columns[4].1 = !self.log_columns[4].1,
                KeyCode::Char('s') => self.log_columns[5].1 = !self.log_columns[5].1,
                KeyCode::Char('f') => self.log_columns[6].1 = !self.log_columns[6].1,
                KeyCode::Char('p') => self.log_columns[7].1 = !self.log_columns[7].1,
                KeyCode::Char('r') => self.auto_scroll = !self.auto_scroll,
                KeyCode::Enter => {
                    if let Some(current_line) = self.search_lines.get_selected_item() {
                            self.log_lines.navigate_to(current_line.unformat().index.parse().unwrap());
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
