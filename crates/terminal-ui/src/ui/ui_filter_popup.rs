use crate::{
    app::{
        App, INDEX_FILTER_APP, INDEX_FILTER_COLOR, INDEX_FILTER_DATETIME, INDEX_FILTER_FUNCTION,
        INDEX_FILTER_NAME, INDEX_FILTER_PAYLOAD, INDEX_FILTER_SEVERITY, INDEX_FILTER_TIMESTAMP,
        INDEX_FILTER_TYPE,
    },
    styles::SELECTED_STYLE,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Tabs,
    },
    Frame, Terminal,
};

use super::{ui_popup::centered_rect, ui_shared::display_cursor};

fn draw_input_field<B>(f: &mut Frame<B>, app: &mut App, area: Rect, index: usize, title: &str)
where
    B: Backend,
{
    let input_widget = Paragraph::new(app.input_buffers[index].value())
        .style(match index == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input_widget, area);
    if index == app.input_buffer_index {
        display_cursor(f, area, app.input_buffers[index].cursor())
    }
}

fn draw_filter_type_selector<B>(
    f: &mut Frame<B>,
    app: &mut App,
    area: Rect,
    index: usize,
    title: &str,
) where
    B: Backend,
{
    let titles = ["INCLUDE", "EXCLUDE", "MARKER"]
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();

    let source_type_widget = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(title))
        .select(app.filter_type)
        .style(match index == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .highlight_style(SELECTED_STYLE);

    f.render_widget(source_type_widget, area);
}

fn draw_color_selector<B>(f: &mut Frame<B>, app: &mut App, area: Rect, index: usize, title: &str)
where
    B: Backend,
{
    let colors = [
        Color::LightYellow,
        Color::Yellow,
        Color::LightRed,
        Color::Red,
        Color::LightGreen,
        Color::Green,
        Color::LightCyan,
        Color::Cyan,
        Color::LightBlue,
        Color::Blue,
        Color::LightMagenta,
        Color::Magenta,
        Color::Black,
        Color::DarkGray,
        Color::Gray,
    ];

    let choices: Vec<Spans> = colors
        .iter()
        .map(|c| {
            Spans::from(vec![
                Span::styled("|X|", Style::default().bg(*c).fg(*c)),
            ])
        })
        .collect();

    let source_type_widget = Tabs::new(choices)
        .block(Block::default().borders(Borders::ALL).title(title))
        .select(app.filter_color)
        .style(match index == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .highlight_style(SELECTED_STYLE.fg(Color::White));

    f.render_widget(source_type_widget, area);
}

pub fn draw_filter_popup<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let block = Block::default()
        .title("Filter")
        .borders(Borders::ALL)
        .border_style(SELECTED_STYLE);

    let area = centered_rect(60, 35, f.size());
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(area);
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(3),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(popup_layout[0]);

    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_NAME - INDEX_FILTER_NAME],
        INDEX_FILTER_NAME,
        "Name",
    );
    draw_filter_type_selector(
        f,
        app,
        popup_layout[INDEX_FILTER_TYPE - INDEX_FILTER_NAME],
        INDEX_FILTER_TYPE,
        "Type",
    );
    draw_color_selector(
        f,
        app,
        popup_layout[INDEX_FILTER_COLOR - INDEX_FILTER_NAME],
        INDEX_FILTER_COLOR,
        "Color",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_DATETIME - INDEX_FILTER_NAME],
        INDEX_FILTER_DATETIME,
        "Datetime",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_TIMESTAMP - INDEX_FILTER_NAME],
        INDEX_FILTER_TIMESTAMP,
        "Timestamp",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_APP - INDEX_FILTER_NAME],
        INDEX_FILTER_APP,
        "App",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_SEVERITY - INDEX_FILTER_NAME],
        INDEX_FILTER_SEVERITY,
        "Severity",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_FUNCTION - INDEX_FILTER_NAME],
        INDEX_FILTER_FUNCTION,
        "Function",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_PAYLOAD - INDEX_FILTER_NAME],
        INDEX_FILTER_PAYLOAD,
        "Payload",
    );
}
