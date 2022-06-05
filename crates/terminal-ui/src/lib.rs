pub mod app;
pub mod styles;
pub mod ui;
pub mod data;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log_analyzer::{
    models::settings::Settings,
    services::log_service::{LogAnalyzer, LogService},
    stores::{
        analysis_store::InMemmoryAnalysisStore, log_store::InMemmoryLogStore,
        processing_store::InMemmoryProcessingStore,
    },
};

use std::{
    error::Error,
    fs, io,
    sync::Arc,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal, style::Color,
};
use ui::{
    ui_error_message::draw_error_popup, ui_filter_popup::draw_filter_popup,
    ui_loading_popup::draw_loading_popup, ui_log_analyzer::draw_log_analyzer_view,
    ui_navigation_popup::draw_navigation_popup, ui_source_popup::draw_source_popup,
};


pub async fn async_main(settings_path: Option<String>) -> Result<(), Box<dyn Error>> {
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
    let mut color = Color::LightBlue;

    if let Some(settings) = settings_path {
        if let Ok(file) = fs::read_to_string(settings) {
            if let Ok(settings) = Settings::from_json(&file) {
                if let Some(formats) = settings.formats {
                    for format in formats {
                        log_service.add_format(&format.alias, &format.regex)?;
                    }
                }
                if let Some(filters) = settings.filters {
                    for filter in filters {
                        log_service.add_filter(filter);
                    }
                }
                if let Some((r, g, b)) = settings.primary_color {
                    color = Color::Rgb(r, g, b)
                }
            }
        }
    }

    // create app and run it
    let tick_rate = Duration::from_millis(150);
    let app = App::new(Box::new(log_service), color).await;
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
                            _ => app.handle_input(key).await,
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
    } else if app.show_filter_popup {
        draw_filter_popup(f, app)
    } else if app.show_navigation_popup {
        draw_navigation_popup(f, app)
    }

    if app.show_error_message {
        draw_error_popup(f, app)
    }

    if app.processing.is_processing {
        draw_loading_popup(f, app)
    }
}
