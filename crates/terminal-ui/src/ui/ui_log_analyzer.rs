use log_analyzer::models::log_line::LogLine;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::{
    app::{App, Module, INDEX_SEARCH},
    styles::selected_style,
};

use super::ui_shared::display_cursor;

trait Convert<T> {
    fn from_str(s: &str) -> Option<T>;
}

impl Convert<Color> for Color {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "BLACK" | "Black" | "black" => Some(Color::Black),
            "WHITE" | "White" | "white" => Some(Color::White),
            "RED" | "Red" | "red" => Some(Color::Red),
            "GREEN" | "Green" | "green" => Some(Color::Green),
            "YELLOW" | "Yellow" | "yellow" => Some(Color::Yellow),
            "BLUE" | "Blue" | "blue" => Some(Color::Blue),
            "MAGENTA" | "Magenta" | "magenta" => Some(Color::Magenta),
            "CYAN" | "Cyan" | "cyan" => Some(Color::Cyan),
            "GRAY" | "Gray" | "gray" => Some(Color::Gray),
            "DARKGRAY" | "DarkGray" | "darkgray" => Some(Color::DarkGray),
            "LIGHTRED" | "LightRed" | "lightred" => Some(Color::LightRed),
            "LIGHTGREEN" | "LightGreen" | "lightgreen" => Some(Color::LightGreen),
            "LIGHTYELLOW" | "LightYellow" | "lightyellow" => Some(Color::LightYellow),
            "LIGHTBLUE" | "LightBlue" | "lightblue" => Some(Color::LightBlue),
            "LIGHTMAGENTA" | "LightMagenta" | "lightmagenta" => Some(Color::LightMagenta),
            "LIGHTCYAN" | "LightCyan" | "lightcyan" => Some(Color::LightCyan),
            _ => None,
        }
    }
}

fn draw_sources<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let sources_widget = Block::default()
        .title("Sources")
        .borders(Borders::ALL)
        .border_style(match app.selected_module {
            Module::Sources => selected_style(app.color),
            _ => Style::default(),
        });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(app.color).add_modifier(Modifier::BOLD);

    let header_cells = ["Enabled", "Log", "Format"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells).style(normal_style).bottom_margin(1);
    let rows = app.sources.items.iter().map(|item| {
        let get_enabled_widget = |enabled: bool| match enabled {
            true => Span::styled("V", Style::default().fg(app.color)),
            false => Span::styled("X", Style::default().fg(Color::Gray)),
        };

        let format = match &item.2 {
            Some(format) => format.as_str(),
            _ => "",
        };

        let cells = vec![
            Cell::from(get_enabled_widget(item.0)),
            Cell::from(Text::from(item.1.as_str())),
            Cell::from(Text::from(format)),
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
            Module::Filters => selected_style(app.color),
            _ => Style::default(),
        });
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(app.color).add_modifier(Modifier::BOLD);

    let header_cells = ["Enabled", "Filter"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells).style(normal_style).bottom_margin(1);

    let rows = app.filters.items.iter().map(|item| {
        let get_enabled_widget = |enabled: bool| match enabled {
            true => Span::styled("V", Style::default().fg(app.color)),
            false => Span::styled("X", Style::default().fg(Color::Gray)),
        };

        let cells = vec![
            Cell::from(get_enabled_widget(item.0)),
            Cell::from(Text::from(item.1.as_str())),
        ];
        Row::new(cells).bottom_margin(0)
    });
    let t = Table::new(rows)
        .header(header)
        .block(filters_widget)
        .highlight_style(selected_style)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);
    f.render_stateful_widget(t, area, &mut app.filters.state);
}

fn draw_sidebar<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let left_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(app.log_filter_size_percentage),
                Constraint::Percentage(100 - app.log_filter_size_percentage),
            ]
            .as_ref(),
        )
        .split(area);

    draw_sources(f, app, left_modules[0]);
    draw_filters(f, app, left_modules[1]);
}

fn log_line_cell_builder<'a>(line: &'a LogLine, column: &'a str, offset: usize) -> Cell<'a> {
    Cell::from(Span::styled(
        line.get(column).unwrap().get(offset..).unwrap_or_default(),
        Style::default().fg(if line.color.is_some() {
            Color::Rgb(
                line.color.unwrap().0,
                line.color.unwrap().1,
                line.color.unwrap().2,
            )
        } else {
            Color::Reset
        }),
    ))
}

fn log_search_cell_builder<'a>(line: &'a LogLine, column: &'a str, mut offset: usize) -> Cell<'a> {
    let content = line.get(column).unwrap();
    let groups: Vec<(Option<&str>, &str)> = match serde_json::from_str(content) {
        Ok(groups) => groups,
        Err(_) => vec![(None, content)]
    };

    Cell::from(Spans::from(
        groups
            .into_iter()
            .filter_map(|(highlight, content)| {
                let style = match (line.color.is_some(), highlight.map(Color::from_str)) {
                    (_, Some(Some(color))) => {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    }
                    (true, _) => Style::default().fg(Color::Rgb(
                        line.color.unwrap().0,
                        line.color.unwrap().1,
                        line.color.unwrap().2,
                    )),
                    _ => Style::default(),
                };

                if highlight.is_some() {
                    style.add_modifier(Modifier::BOLD);
                }
                let retval = content.get(offset..).map(|str| Span::styled(str, style));

                offset = offset.saturating_sub(content.len());
                retval
            })
            .collect::<Vec<Span<'a>>>(),
    ))
}

fn draw_log<'a, 's, B>(
    f: &mut Frame<B>,
    app: &'s mut App,
    module: Module,
    title: &str,
    cell_builder: &dyn Fn(&'s LogLine, &'s str, usize) -> Cell<'a>,
    area: Rect,
) where
    B: Backend,
{
    let is_selected = app.selected_module == module;
    let items = if module == Module::Logs {
        &app.log_lines.items
    } else {
        &app.search_lines.items
    };
    let log_widget = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(match is_selected {
            true => selected_style(app.color),
            _ => Style::default(),
        });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(app.color).add_modifier(Modifier::BOLD);

    let enabled_columns: Vec<&(String, bool)> = app
        .log_columns
        .iter()
        .filter(|(_, enabled)| *enabled)
        .collect();

    let header_cells = enabled_columns
        .iter()
        .map(|(column, _)| Cell::from(column.clone()).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells).style(normal_style).bottom_margin(1);

    let rows = items.iter().map(|item| {
        let cells = enabled_columns
            .iter()
            .map(|(column, _)| cell_builder(item, column, app.horizontal_offset));
        Row::new(cells).bottom_margin(0)
    });

    let constraints: Vec<Constraint> = enabled_columns
        .iter()
        .map(|(name, _)| Constraint::Length(app.get_column_lenght(name)))
        .collect();

    let t = Table::new(rows)
        .header(header)
        .block(log_widget)
        .highlight_style(selected_style)
        .widths(&constraints);

    let state = if module == Module::Logs {
        &mut app.log_lines.state
    } else {
        &mut app.search_lines.state
    };
    f.render_stateful_widget(t, area, state);
}

fn draw_search_box<B>(f: &mut Frame<B>, app: &mut App, area: Rect, index: usize, title: &str)
where
    B: Backend,
{
    let input_widget = Paragraph::new(app.input_buffers[index].value())
        .style(match app.selected_module {
            Module::Search => selected_style(app.color),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input_widget, area);

    if app.selected_module == Module::Search {
        display_cursor(f, area, app.input_buffers[index].cursor())
    }
}

fn draw_bottom_bar<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let bottom_bar_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let auto_scroll = Paragraph::new("AUTO SCROLL")
        .style(match app.auto_scroll {
            false => Style::default().add_modifier(Modifier::DIM),
            true => selected_style(app.color),
        })
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(auto_scroll, bottom_bar_layout[0]);

    let total = app.log_analyzer.get_total_raw_lines();
    let filtered = app.log_analyzer.get_total_filtered_lines();
    let label = format!(" {}/{}", filtered, total);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(app.color))
        .percent((if total > 0 { filtered * 100 / total } else { 0 }) as u16)
        .label(label);
    f.render_widget(gauge, bottom_bar_layout[1]);

    let searched = app.log_analyzer.get_total_searched_lines();
    let label = format!(" {}/{}", searched, total);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(app.color))
        .percent((if total > 0 { searched * 100 / total } else { 0 }) as u16)
        .label(label);

    f.render_widget(gauge, bottom_bar_layout[2]);
}

fn draw_main_panel<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let expandable = area.height - 3;
    let log_lenght = expandable * (app.log_search_size_percentage) as u16 / 100;
    let search_lenght = expandable * (100 - app.log_search_size_percentage) as u16 / 100;

    let main_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(log_lenght),
                Constraint::Length(3),
                Constraint::Length(search_lenght),
            ]
            .as_ref(),
        )
        .split(area);

    draw_log(
        f,
        app,
        Module::Logs,
        "Log",
        &log_line_cell_builder,
        main_modules[0],
    );
    draw_search_box(f, app, main_modules[1], INDEX_SEARCH, "Search");
    draw_log(
        f,
        app,
        Module::SearchResult,
        "Search results",
        &log_search_cell_builder,
        main_modules[2],
    );
}

pub fn draw_log_analyzer_view<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let ui = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(f.size().height - 3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Create two chunks with equal horizontal screen space
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.side_main_size_percentage),
            Constraint::Percentage(100 - app.side_main_size_percentage),
        ])
        .split(ui[0]);

    draw_sidebar(f, app, panels[0]);
    draw_main_panel(f, app, panels[1]);
    draw_bottom_bar(f, app, ui[1]);
}
