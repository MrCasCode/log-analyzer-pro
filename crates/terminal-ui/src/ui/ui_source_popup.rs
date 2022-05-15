use crate::{
    app::{
        App, INDEX_SOURCE_FORMAT, INDEX_SOURCE_NEW_FORMAT_ALIAS, INDEX_SOURCE_NEW_FORMAT_REGEX,
        INDEX_SOURCE_OK_BUTTON, INDEX_SOURCE_PATH, INDEX_SOURCE_TYPE,
    },
    styles::SELECTED_STYLE,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

use super::{ui_popup::centered_rect, ui_shared::display_cursor};

fn draw_source_type_selector<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let titles = ["FILE", "WS"]
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();

    let source_type_widget = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Source type"))
        .select(app.source_type)
        .style(match INDEX_SOURCE_TYPE == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .highlight_style(SELECTED_STYLE);

    f.render_widget(source_type_widget, area);
}

fn draw_source_path<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let source_path_widget = Paragraph::new(app.input_buffers[INDEX_SOURCE_PATH].value())
        .style(match INDEX_SOURCE_PATH == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .block(Block::default().borders(Borders::ALL).title("Path"));

    f.render_widget(source_path_widget, area);
    if INDEX_SOURCE_PATH == app.input_buffer_index {
        display_cursor(f, area, app.input_buffers[INDEX_SOURCE_PATH].cursor())
    }
}

fn draw_format_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let formats: Vec<ListItem> = app
        .formats
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.clone())];
            ListItem::new(lines).style(Style::default().fg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let formats = List::new(formats)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Format")
                .border_style(match INDEX_SOURCE_FORMAT == app.input_buffer_index {
                    false => Style::default(),
                    true => SELECTED_STYLE,
                }),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(formats, area, &mut app.formats.state);
}

fn draw_new_format_alias<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let format_alias_widget =
        Paragraph::new(app.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS].value())
            .style(
                match INDEX_SOURCE_NEW_FORMAT_ALIAS == app.input_buffer_index {
                    false => Style::default(),
                    true => SELECTED_STYLE,
                },
            )
            .block(Block::default().borders(Borders::ALL).title("Alias"));

    f.render_widget(format_alias_widget, area);
    if INDEX_SOURCE_NEW_FORMAT_ALIAS == app.input_buffer_index {
        display_cursor(f, area, app.input_buffers[INDEX_SOURCE_NEW_FORMAT_ALIAS].cursor())
    }
}

fn draw_new_format_regex<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let format_regex_widget =
        Paragraph::new(app.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX].value())
            .style(
                match INDEX_SOURCE_NEW_FORMAT_REGEX == app.input_buffer_index {
                    false => Style::default(),
                    true => SELECTED_STYLE,
                },
            )
            .block(Block::default().borders(Borders::ALL).title("Regex"));

    f.render_widget(format_regex_widget, area);
    if INDEX_SOURCE_NEW_FORMAT_REGEX == app.input_buffer_index {
        display_cursor(f, area, app.input_buffers[INDEX_SOURCE_NEW_FORMAT_REGEX].cursor())
    }
}

fn draw_ok_button<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let ok_button_widget = Paragraph::new("OK")
        .style(match INDEX_SOURCE_OK_BUTTON == app.input_buffer_index {
            false => Style::default(),
            true => SELECTED_STYLE,
        })
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(ok_button_widget, area);
}

pub fn draw_source_popup<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let block = Block::default()
        .title("Add new source")
        .borders(Borders::ALL)
        .border_style(SELECTED_STYLE);

    let area = centered_rect(60, 28, f.size());
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
                Constraint::Percentage(40),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(popup_layout[0]);

    draw_source_type_selector(f, app, popup_layout[INDEX_SOURCE_TYPE]);
    draw_source_path(f, app, popup_layout[INDEX_SOURCE_PATH]);
    draw_format_list(f, app, popup_layout[INDEX_SOURCE_FORMAT]);
    draw_new_format_alias(f, app, popup_layout[INDEX_SOURCE_NEW_FORMAT_ALIAS]);
    draw_new_format_regex(f, app, popup_layout[INDEX_SOURCE_NEW_FORMAT_REGEX]);
    draw_ok_button(f, app, popup_layout[INDEX_SOURCE_OK_BUTTON]);
}
