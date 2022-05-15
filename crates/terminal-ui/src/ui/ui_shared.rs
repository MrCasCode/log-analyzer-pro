use tui::{backend::Backend, Frame, layout::Rect};

pub fn display_cursor<B>(f: &mut Frame<B>, area: Rect, cursor: usize)
where
    B: Backend,
{
    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    f.set_cursor(
        // Put cursor past the end of the input text
        area.x + (cursor as u16).min(area.width.max(3) - 3) + 1,
        // Move one line down, from the border to the input line
        area.y + 1,
    )
}