use log_analyzer::models::log_line::LogLine;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::{
    app::{Module, StatefulTable, INDEX_SEARCH},
    styles::{SELECTED_COLOR, SELECTED_STYLE},
    App,
};

use super::ui_shared::display_cursor;

fn draw_sources<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let sources_widget = Block::default()
        .title("Sources")
        .borders(Borders::ALL)
        .border_style(match app.selected_module {
            Module::Sources => SELECTED_STYLE,
            _ => Style::default(),
        });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default()
        .bg(SELECTED_COLOR)
        .add_modifier(Modifier::BOLD);

    let header_cells = ["Enabled", "Log", "Format"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells).style(normal_style).bottom_margin(1);
    let r = app.sources.items.read().unwrap();
    let rows = r.iter().map(|item| {
        let get_enabled_widget = |enabled: bool| match enabled {
            true => Span::styled("V", Style::default().fg(SELECTED_COLOR)),
            false => Span::styled("X", Style::default().fg(Color::Gray)),
        };

        let cells = vec![
            Cell::from(get_enabled_widget(item.0)),
            Cell::from(Text::from(item.1.as_str())),
            Cell::from(Text::from(item.2.as_str())),
        ];
        Row::new(cells).bottom_margin(0)
    });
    let t = Table::new(rows)
        .header(header)
        .block(sources_widget)
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ]);
    f.render_stateful_widget(t, area, &mut app.sources.state);
}

fn draw_filters<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let filters_widget = Block::default()
        .title("Filters")
        .borders(Borders::ALL)
        .border_style(match app.selected_module {
            Module::Filters => SELECTED_STYLE,
            _ => Style::default(),
        });
    f.render_widget(filters_widget, area);
}

fn draw_sidebar<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let left_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    draw_sources(f, app, left_modules[0]);
    draw_filters(f, app, left_modules[1]);
}

fn draw_log<B>(
    f: &mut Frame<B>,
    is_selected: bool,
    items: &mut StatefulTable<LogLine>,
    log_columns: &Vec<(String, bool)>,
    title: &str,
    horizontal_offset: usize,
    area: Rect,
) where
    B: Backend,
{
    let log_widget = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(match is_selected {
            true => SELECTED_STYLE,
            _ => Style::default(),
        });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default()
        .bg(SELECTED_COLOR)
        .add_modifier(Modifier::BOLD);

    let enabled_columns: Vec<&(String, bool)> =
        log_columns.iter().filter(|(_, enabled)| *enabled).collect();

    let header_cells = enabled_columns
        .iter()
        .map(|(column, _)| Cell::from(column.clone()).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells).style(normal_style).bottom_margin(1);

    let r = items.items.read().unwrap();
    let rows = r.iter().map(|item| {
        let cells = enabled_columns.iter().map(|(column, _)| {
            Cell::from(Span::styled(
                item.get(&column)
                    .unwrap()
                    .get(horizontal_offset..)
                    .unwrap_or_default(),
                Style::default(),
            ))
        });
        Row::new(cells).bottom_margin(0)
    });

    let mut constraints = vec![Constraint::Min(15); enabled_columns.len() - 1];
    constraints.push(Constraint::Percentage(100));

    let t = Table::new(rows)
        .header(header)
        .block(log_widget)
        .highlight_style(selected_style)
        .widths(&constraints);
    f.render_stateful_widget(t, area, &mut items.state);
}

fn draw_search_box<B>(f: &mut Frame<B>, app: &mut App, area: Rect, index: usize, title: &str)
where
    B: Backend,
{
    let input_widget = Paragraph::new(app.input_buffers[index].value())
        .style(match app.selected_module {
            Module::Search => SELECTED_STYLE,
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input_widget, area);

    if app.selected_module == Module::Search {
        display_cursor(f, area, app.input_buffers[index].cursor())
    }
}

fn draw_main_panel<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let main_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(75),
                Constraint::Max(3),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(area);

    draw_log(
        f,
        app.selected_module == Module::Logs,
        &mut app.log_lines,
        &mut app.log_columns,
        "Log",
        app.horizontal_offset,
        main_modules[0],
    );
    draw_search_box(f, app, main_modules[1], INDEX_SEARCH, "Search");
    draw_log(
        f,
        app.selected_module == Module::SearchResult,
        &mut app.search_lines,
        &mut app.log_columns,
        "Search results",
        app.horizontal_offset,
        main_modules[2],
    );
}

pub fn draw_log_analyzer_view<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let constraints = if app.show_side_panel {
        [Constraint::Percentage(25), Constraint::Percentage(75)].as_ref()
    } else {
        [Constraint::Percentage(0), Constraint::Percentage(100)].as_ref()
    };
    // Create two chunks with equal horizontal screen space
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(f.size());

    draw_sidebar(f, app, panels[0]);
    draw_main_panel(f, app, panels[1])
}
