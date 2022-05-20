mod app;
mod styles;
mod ui;

use app::{
    App, INDEX_SOURCE_FORMAT, INDEX_SOURCE_NEW_FORMAT_ALIAS, INDEX_SOURCE_NEW_FORMAT_REGEX, INDEX_SOURCE_OK_BUTTON, INDEX_SOURCE_PATH,
    INDEX_SOURCE_TYPE,
};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log_analyzer::{stores::{log_store::InMemmoryLogStore, processing_store::{InMemmoryProcessingStore, ProcessingStore}, analysis_store::InMemmoryAnalysisStore}, services::log_service::{LogService, LogAnalyzer}, models::settings::Settings};

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    io,
    slice::Iter,
    time::{Duration, Instant}, sync::Arc,
    fs
};
use styles::SELECTED_STYLE;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use ui::{ui_log_analyzer::draw_log_analyzer_view, ui_source_popup::draw_source_popup, ui_filter_popup::draw_filter_popup, ui_error_message::draw_error_popup};




#[derive(Debug, PartialEq)]
pub enum Panel {
    /// Left panel where sources and filters are located
    Left,
    /// Main panel where logs are displayed
    Main,
}



async fn async_main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create
    let log_store = Arc::new(InMemmoryLogStore::new());
    let processing_store = Arc::new(InMemmoryProcessingStore::new());
    let analysis_store = Arc::new(InMemmoryAnalysisStore::new());

    let log_service = LogService::new(log_store, processing_store, analysis_store);

    if let Ok(file) = fs::read_to_string("settings.json") {
        if let Ok(settings) = Settings::from_json(&file) {
            for format in settings.formats {
                log_service.add_format(&format.alias, &format.regex);
            }
            for filter in settings.filters {
                //processing_store.add_filter(filter.alias, filter.filter, filter.action, false);
            }
        }

    }




    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(Box::new(log_service)).await;
    let res = run_app(&mut terminal, app, tick_rate).await;


    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    async_std::task::block_on(async_main())?;

    Ok(())
}




async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            let event = event::read()?;

            match event {
                Event::Key(key) => {
                    match key.modifiers {
                        // Quit
                        KeyModifiers::CONTROL => match key.code {
                            KeyCode::Char('c') => return Ok(()),
                            KeyCode::Char('s') => app.show_side_panel = !app.show_side_panel,
                            _ => async_std::task::block_on(app.handle_input(key)),
                        },
                        // Navigate
                        KeyModifiers::SHIFT => match key.code {
                            KeyCode::Char(_) => async_std::task::block_on(app.handle_input(key)),
                            KeyCode::Up | KeyCode::BackTab => app.navigate(KeyCode::Up),
                            KeyCode::Down | KeyCode::Tab => app.navigate(KeyCode::Down),
                            KeyCode::Left => app.navigate(KeyCode::Left),
                            KeyCode::Right => app.navigate(KeyCode::Right),
                            _ => {}
                        },
                        // Handle in widget
                        _ => match key.code {
                            KeyCode::Tab => app.navigate(KeyCode::Down),
                            _ => async_std::task::block_on(app.handle_input(key)),
                        },
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {}
                    MouseEventKind::ScrollDown => {}
                    MouseEventKind::Down(button) => match button {
                        crossterm::event::MouseButton::Left => {}
                        crossterm::event::MouseButton::Right => {}
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    draw_log_analyzer_view(f, app);

    if app.show_source_popup {
        draw_source_popup(f, app)
    }
    else if app.show_filter_popup {
        draw_filter_popup(f, app)
    }

    if app.show_error_message {
        draw_error_popup(f, app)
    }
}