use crate::{
    app::{App, INDEX_NAVIGATION},
    styles::selected_style,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::{ui_popup::centered_rect, ui_shared::display_cursor};

fn draw_navigation_input(f: &mut Frame, app: &App, area: Rect) {
    let format_regex_widget = Paragraph::new(app.input_buffers[INDEX_NAVIGATION].value())
        .style(selected_style(app.color))
        .block(Block::default().borders(Borders::ALL).title("Index"));

    f.render_widget(format_regex_widget, area);
    if INDEX_NAVIGATION == app.input_buffer_index {
        display_cursor(f, area, app.input_buffers[INDEX_NAVIGATION].cursor())
    }
}

pub fn draw_navigation_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Navigate to index")
        .borders(Borders::ALL)
        .border_style(selected_style(app.color));

    let area = centered_rect(60, 7, f.size());
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(area);
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)].as_ref())
        .margin(1)
        .split(popup_layout[0]);

    draw_navigation_input(f, app, popup_layout[0]);
}
