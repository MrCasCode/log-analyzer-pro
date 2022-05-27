use tui::style::{Color, Modifier, Style};


pub fn selected_style(selected_color: Color) -> Style {
    Style {
        fg: Some(selected_color),
        bg: None,
        add_modifier: Modifier::BOLD,
        sub_modifier: Modifier::empty(),
    }
}

pub const ERROR_STYLE: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
