use anyhow::{anyhow, Result};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    MouseEvent, MouseEventKind,
};
use log_analyzer::services::{log_service::LogAnalyzer, log_source::SourceType};
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    future::Future,
    io,
    slice::Iter,
    sync::{mpsc::channel, WaitTimeoutResult},
    task::Poll,
    time::{Duration, Instant},
};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{ListState, TableState},
};

use async_std::{
    channel::{self, Receiver, Sender},
    sync::Condvar,
};

/* ------ NEW SOURCE INDEXES ------- */
pub const INDEX_SOURCE_TYPE: usize = 0;
pub const INDEX_SOURCE_PATH: usize = 1;
pub const INDEX_SOURCE_FORMAT: usize = 2;
pub const INDEX_SOURCE_NEW_FORMAT_ALIAS: usize = 3;
pub const INDEX_SOURCE_NEW_FORMAT_REGEX: usize = 4;
pub const INDEX_SOURCE_OK_BUTTON: usize = 5;
/* ----------------------------------- */

/* ------ FILTER INDEXES ------- */
pub const INDEX_FILTER_NAME: usize = 0;
pub const INDEX_FILTER_TYPE: usize = 1;
pub const INDEX_FILTER_COLOR: usize = 2;
pub const INDEX_FILTER_DATETIME: usize = 3;
pub const INDEX_FILTER_TIMESTAMP: usize = 4;
pub const INDEX_FILTER_APP: usize = 5;
pub const INDEX_FILTER_SEVERITY: usize = 6;
pub const INDEX_FILTER_FUNCTION: usize = 7;
pub const INDEX_FILTER_PAYLOAD: usize = 8;
pub const INDEX_FILTER_OK_BUTTON: usize = 9;
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
    SourcePopup,
    FilterPopup,
    ErrorPopup,
    None,
}

/* Supported directions of scrolling */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
}

impl ScrollDirection {
    /**
     * Return iterator of the available scroll directions.
     *
     * @return Iter
     */
    #[allow(dead_code)]
    pub fn iter() -> Iter<'static, ScrollDirection> {
        [
            ScrollDirection::Up,
            ScrollDirection::Down,
            ScrollDirection::Left,
            ScrollDirection::Right,
            ScrollDirection::Top,
            ScrollDirection::Bottom,
        ]
        .iter()
    }
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub struct App<'a> {
    pub log_analyzer: Box<dyn LogAnalyzer>,

    pub selected_module: Module,

    pub show_side_panel: bool,
    pub show_source_popup: bool,
    pub show_filter_popup: bool,

    pub input_buffers: Vec<String>,
    pub input_buffer_index: usize,
    pub formats: StatefulList<String>,

    /// Tab selector index for Source Type
    pub source_type: usize,
    /// Tab selector index for Filter Type
    pub filter_type: usize,
    /// Tab selector index for Filter Type
    pub filter_color: usize,

    // Display all log sources in the sources panel
    pub sources: StatefulTable<(bool, String, String)>,

    pub items: StatefulList<(&'a str, usize)>,
    pub events: Vec<(&'a str, &'a str)>,

    pub show_error_message: bool,

    pub popup: PopupInteraction,
}

impl<'a> App<'a> {
    pub async fn new(log_analyzer: Box<dyn LogAnalyzer>) -> App<'a> {
        let items = vec![("Item0", 1); 50];

        let mut formats = vec!["New".to_string()];
        formats.extend(log_analyzer.get_formats().await);

        let sources = log_analyzer.get_logs().await;

        App {
            log_analyzer,
            selected_module: Module::Sources,
            show_side_panel: true,
            show_source_popup: false,
            show_filter_popup: false,

            input_buffers: vec![String::new(); 20],
            input_buffer_index: 0,

            formats: StatefulList::with_items(formats),

            source_type: 0,
            filter_type: 0,
            filter_color: 0,

            sources: StatefulTable::with_items(sources),

            items: StatefulList::with_items(items),
            events: vec![
                ("Event1", "INFO"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
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

        let alias: String;
        // New
        if selected_format_index == 0 {
            alias = self.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS].clone();
            let regex = self.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX].clone();

            self.log_analyzer.add_format(&alias, &regex).await?;
            self.update_formats().await;
        } else {
            alias = self.formats.items[selected_format_index].clone();
        }

        let path = self.input_buffers[INDEX_SOURCE_PATH].clone();
        self.log_analyzer
            .add_log(self.source_type, &path, &alias)
            .await?;

        Ok(())
    }

    pub async fn update_formats(&mut self) {
        let mut formats = vec!["New".to_string()];
        formats.extend(self.log_analyzer.get_formats().await);

        self.formats = StatefulList::with_items(formats);
        self.formats.state.select(Some(0));
    }

    pub async fn update_sources(&mut self) {
        let sources = self.log_analyzer.get_logs().await;
        self.sources = StatefulTable::with_items(sources)
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    pub fn on_tick(&mut self) {
        let event = self.events.remove(0);
        self.events.push(event);
    }

    pub async fn handle_input(&mut self, key: KeyEvent) {
        match self.selected_module {
            Module::Sources => self.handle_sources_input(key).await,
            Module::Filters => self.handle_filters_input(key).await,
            Module::Logs => self.handle_sources_input(key).await,
            Module::Search => self.handle_sources_input(key).await,
            Module::SourcePopup => self.handle_source_popup_input(key).await,
            Module::FilterPopup => self.handle_filter_popup_input(key).await,
            Module::ErrorPopup => self.handle_error_popup_input(key).await,
            _ => {}
        }
    }

    async fn handle_sources_input(&mut self, key: KeyEvent) {
        match key.code {
            // Navigate up sources
            KeyCode::Up => {}
            // Navigate down sources
            KeyCode::Down => {}
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

    async fn handle_source_popup_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.input_buffers[self.input_buffer_index].push(c);
            }
            KeyCode::Backspace => {
                if self.input_buffer_index == INDEX_SOURCE_FORMAT {
                    // TODO: Delete format
                }
                self.input_buffers[self.input_buffer_index].pop();
            }

            // Navigate up sources
            KeyCode::Up => {
                if self.input_buffer_index == INDEX_SOURCE_FORMAT {
                    self.formats.previous();
                }
            }
            // Navigate down sources
            KeyCode::Down => {
                if self.input_buffer_index == INDEX_SOURCE_FORMAT {
                    self.formats.next();
                }
            }
            // Switch between file and ws
            KeyCode::Right | KeyCode::Left => {
                if self.input_buffer_index == INDEX_SOURCE_TYPE {
                    self.source_type = !self.source_type & 1;
                }
            }
            KeyCode::Enter => {
                if self.input_buffer_index == INDEX_SOURCE_OK_BUTTON {
                    match self.add_log().await {
                        Ok(_) => {
                            self.show_source_popup = false;
                            self.selected_module = Module::Sources;
                            self.update_sources().await
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
            // Add new source -> Popup window
            KeyCode::Esc => {
                self.show_source_popup = false;
                self.selected_module = Module::Sources;
            }
            // Nothing
            _ => {}
        }
    }

    async fn handle_filter_popup_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.input_buffers[self.input_buffer_index].push(c);
            }
            KeyCode::Backspace => {
                self.input_buffers[self.input_buffer_index].pop();
            }

            KeyCode::Up => {}

            KeyCode::Down => {}

            // Switch tabs
            KeyCode::Right | KeyCode::Left => {
                let circular_choice = |i: &mut usize, max, add: i32| {
                    *i = match (*i as i32 + add) as i32 {
                        r if r > max => 0 as usize, // if adding overflows -> set to 0
                        r if r < 0 => max as usize, // if adding underflows -> set to 0
                        r => r as usize,
                    }
                };

                let sum = if key.code == KeyCode::Right { 1 } else { -1 };
                match self.input_buffer_index {
                    INDEX_FILTER_TYPE => circular_choice(&mut self.filter_type, 2, sum),
                    INDEX_FILTER_COLOR => circular_choice(&mut self.filter_color, 14, sum),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                if self.input_buffer_index == INDEX_SOURCE_OK_BUTTON {
                    // TODO: Validate args and show error message if error
                    // TODO: Add format if new
                    // TODO: Add log
                }
            }
            // Add new source -> Popup window
            KeyCode::Esc => {
                self.show_filter_popup = false;
                self.selected_module = Module::Filters;
            }
            // Nothing
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

    async fn handle_filters_input(&mut self, key: KeyEvent) {
        match key.code {
            // Navigate up sources
            KeyCode::Up => {}
            // Navigate down sources
            KeyCode::Down => {}
            // Toggle enabled/disabled source
            KeyCode::Enter => {}
            // Add new source -> Popup window
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

    pub fn navigate(&mut self, direction: KeyCode) {
        match self.selected_module {
            Module::Sources => match direction {
                KeyCode::Up | KeyCode::Down => self.selected_module = Module::Filters,
                KeyCode::Left | KeyCode::Right => self.selected_module = Module::Logs,
                _ => {}
            },
            Module::Filters => match direction {
                KeyCode::Up | KeyCode::Down => self.selected_module = Module::Sources,
                KeyCode::Left | KeyCode::Right => self.selected_module = Module::Search,
                _ => {}
            },
            Module::Logs => match direction {
                KeyCode::Up | KeyCode::Down => self.selected_module = Module::Search,
                KeyCode::Left | KeyCode::Right => {
                    if self.show_side_panel {
                        self.selected_module = Module::Sources
                    }
                }
                _ => {}
            },
            Module::Search => match direction {
                KeyCode::Up | KeyCode::Down => self.selected_module = Module::Logs,
                KeyCode::Left | KeyCode::Right => {
                    if self.show_side_panel {
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
}
