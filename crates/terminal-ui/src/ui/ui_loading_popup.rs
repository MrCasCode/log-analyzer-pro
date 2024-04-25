use crate::{app::App, styles::selected_style};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::ui_popup::centered_rect;

fn draw_navigation_input(f: &mut Frame, app: &App, area: Rect) {
    let format_regex_widget = Paragraph::new("...")
        .style(selected_style(app.color))
        .block(Block::default().borders(Borders::ALL).title("Loading"));

    f.render_widget(format_regex_widget, area);
}

pub fn draw_loading_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
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
