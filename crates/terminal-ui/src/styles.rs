use tui::style::{Style, Modifier, Color};

pub const SELECTED_COLOR: Color = Color::LightBlue;

pub const SELECTED_STYLE: Style = Style {
    fg: Some(SELECTED_COLOR),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const ERROR_STYLE: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};