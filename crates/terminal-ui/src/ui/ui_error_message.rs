use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::App,
    styles::{selected_style, ERROR_STYLE},
};

use super::ui_popup::centered_rect;

fn draw_error_message(f: &mut Frame, app: &mut App, area: Rect) {
    let ok_button_widget = Paragraph::new(app.popup.message.as_str())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(ok_button_widget, area);
}

fn draw_ok_button(f: &mut Frame, app: &App, area: Rect) {
    let ok_button_widget = Paragraph::new("OK")
        .style(selected_style(app.color))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(ok_button_widget, area);
}

pub fn draw_error_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .border_style(ERROR_STYLE);

    let area = centered_rect(30, 15, f.size());
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(area);
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .margin(1)
        .split(popup_layout[0]);

    draw_error_message(f, app, popup_layout[0]);
    draw_ok_button(f, app, popup_layout[1]);
}
