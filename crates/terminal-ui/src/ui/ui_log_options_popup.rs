use crate::{
    app::{
        App, INDEX_FILTER_APP, INDEX_FILTER_BLUE_COLOR, INDEX_FILTER_DATETIME,
        INDEX_FILTER_FUNCTION, INDEX_FILTER_GREEN_COLOR, INDEX_FILTER_NAME, INDEX_FILTER_OK_BUTTON,
        INDEX_FILTER_PAYLOAD, INDEX_FILTER_RED_COLOR, INDEX_FILTER_SEVERITY,
        INDEX_FILTER_TIMESTAMP, INDEX_FILTER_TYPE, parse_color,
    },
    styles::selected_style,
};
use tui::{
    backend::Backend,
    layout::{Alignment, Layout, Rect, Constraint, Direction},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Clear, Paragraph, Tabs,
    },
    Frame,
};

use super::{ui_popup::centered_rect, ui_shared::display_cursor};

fn draw_input_field<B>(f: &mut Frame<B>, app: &mut App, area: Rect, index: usize, title: &str)
where
    B: Backend,
{
    let input_widget = Paragraph::new(app.input_buffers[index].value())
        .style(match index == app.input_buffer_index {
            false => Style::default(),
            true => selected_style(app.color),
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
            true => selected_style(app.color),
        })
        .highlight_style(selected_style(app.color));

    f.render_widget(source_type_widget, area);
}

fn draw_color_selector<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let color_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(60),
                Constraint::Max(3),
            ]
            .as_ref(),
        )
        .margin(0)
        .split(area);
    draw_input_field(f, app, color_layout[0], INDEX_FILTER_RED_COLOR, "Red");
    draw_input_field(f, app, color_layout[1], INDEX_FILTER_GREEN_COLOR, "Green");
    draw_input_field(f, app, color_layout[2], INDEX_FILTER_BLUE_COLOR, "Blue");

    let w = Block::default().borders(Borders::ALL);
    let color = match parse_color(
        app.input_buffers[INDEX_FILTER_RED_COLOR].value(),
        app.input_buffers[INDEX_FILTER_GREEN_COLOR].value(),
        app.input_buffers[INDEX_FILTER_BLUE_COLOR].value(),
    ) {
        Some((r, g, b)) => Color::Rgb(r, g, b),
        _ => Color::Reset,
    };

    //let (r, g, b) = app.input_buffers[INDEX_FILTER_RED_COLOR].value().parse(), app.input_buffers[INDEX_FILTER_GREEN_COLOR].value(), app.input_buffers[INDEX_FILTER_BLUE_COLOR].value()
    let w_color = Paragraph::new(if color == Color::Reset {
        "No color"
    } else {
        ""
    })
    .block(Block::default().style(Style::default().bg(color)));

    f.render_widget(w_color, w.inner(color_layout[3]));
}

fn draw_separator<B>(f: &mut Frame<B>, title: &str, area: Rect, offset: &mut usize)
where
    B: Backend,
{
    *offset += 1;
    f.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .title(title)
            .title_alignment(Alignment::Center),
        area,
    );
}

fn draw_ok_button<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let ok_button_widget = Paragraph::new("OK")
        .style(match INDEX_FILTER_OK_BUTTON == app.input_buffer_index {
            false => Style::default(),
            true => selected_style(app.color),
        })
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(ok_button_widget, area);
}

pub fn draw_log_options_popup<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let mut offset = 0_usize;
    let block = Block::default()
        .title("Log Options")
        .borders(Borders::ALL)
        .border_style(selected_style(app.color));

    let area = centered_rect(60, 36, f.size());
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
                Constraint::Length(1)
            ]
            .as_ref(),
        )
        .margin(1)
        .split(popup_layout[0]);

    draw_input_field(
        f,
        app,
        popup_layout[0],
        INDEX_FILTER_NAME,
        "Name",
    );
    draw_filter_type_selector(
        f,
        app,
        popup_layout[INDEX_FILTER_TYPE - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_TYPE,
        "Type",
    );

    draw_separator(
        f,
        "Filter",
        popup_layout[INDEX_FILTER_DATETIME - INDEX_FILTER_NAME + offset],
        &mut offset,
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_DATETIME - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_DATETIME,
        "Datetime",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_TIMESTAMP - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_TIMESTAMP,
        "Timestamp",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_APP - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_APP,
        "App",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_SEVERITY - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_SEVERITY,
        "Severity",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_FUNCTION - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_FUNCTION,
        "Function",
    );
    draw_input_field(
        f,
        app,
        popup_layout[INDEX_FILTER_PAYLOAD - INDEX_FILTER_NAME + offset],
        INDEX_FILTER_PAYLOAD,
        "Payload",
    );
    draw_separator(
        f,
        "Color",
        popup_layout[INDEX_FILTER_RED_COLOR - INDEX_FILTER_NAME + offset],
        &mut offset,
    );
    draw_color_selector(
        f,
        app,
        popup_layout[INDEX_FILTER_RED_COLOR - INDEX_FILTER_NAME + offset],
    );
    draw_ok_button(f, app, popup_layout[popup_layout.len() - 1])
}
