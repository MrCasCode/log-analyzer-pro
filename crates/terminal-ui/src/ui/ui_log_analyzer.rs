use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::{
    app::Module,
    styles::{SELECTED_COLOR, SELECTED_STYLE},
    App,
};

fn draw_sources<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let sources_widget = Block::default()
        .title("Sources")
        .borders(Borders::ALL)
        .border_style(match app.selected_module {
            Module::Sources => SELECTED_STYLE,
            _ => Style::default(),
        });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(SELECTED_COLOR).add_modifier(Modifier::BOLD);

    let header_cells = ["Enabled", "Log", "Format"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .bottom_margin(1);

    let rows = app.sources.items.iter().map(|item| {
        let get_enabled_widget = |enabled: bool| match enabled {
            true => Span::styled("V", Style::default().fg(SELECTED_COLOR)),
            false => Span::styled("X", Style::default().fg(Color::Gray)),
        };

        let cells = vec![
            Cell::from(get_enabled_widget(item.0)),
            Cell::from(Text::from(item.1.as_str())),
            Cell::from(Text::from(item.2.as_str())),
        ];
        Row::new(cells).bottom_margin(1)
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
            Module::Filters => SELECTED_STYLE,
            _ => Style::default(),
        });
    f.render_widget(filters_widget, area);
}

fn draw_sidebar<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let left_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    draw_sources(f, app, left_modules[0]);
    draw_filters(f, app, left_modules[1]);
}

fn draw_main_panel<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let main_modules = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(area);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.0)];
            ListItem::new(lines).style(Style::default().fg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Log")
                .border_style(match app.selected_module {
                    Module::Logs => SELECTED_STYLE,
                    _ => Style::default(),
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, main_modules[0], &mut app.items.state);

    // Let's do the same for the events.
    // The event list doesn't have any state and only displays the current state of the list.
    let events: Vec<ListItem> = app
        .events
        .iter()
        .rev()
        .map(|&(event, level)| {
            // Colorcode the level depending on its type
            let s = match level {
                "CRITICAL" => Style::default().fg(Color::Red),
                "ERROR" => Style::default().fg(Color::Magenta),
                "WARNING" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };
            // Add a example datetime and apply proper spacing between them
            let header = Spans::from(vec![
                Span::styled(format!("{:<9}", level), s),
                Span::raw(" "),
                Span::styled(
                    "2020-01-01 10:00:00",
                    Style::default().add_modifier(Modifier::ITALIC),
                ),
            ]);
            // The event gets its own line
            let log = Spans::from(vec![Span::raw(event)]);

            // Here several things happen:
            // 1. Add a `---` spacing line above the final list entry
            // 2. Add the Level + datetime
            // 3. Add a spacer line
            // 4. Add the actual event
            ListItem::new(vec![
                Spans::from("-".repeat(main_modules[1].width as usize)),
                header,
                Spans::from(""),
                log,
            ])
        })
        .collect();
    let events_list = List::new(events)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(match app.selected_module {
                    Module::Search => SELECTED_STYLE,
                    _ => Style::default(),
                }),
        )
        .start_corner(Corner::BottomLeft);
    f.render_widget(events_list, main_modules[1]);
}

pub fn draw_log_analyzer_view<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // Create two chunks with equal horizontal screen space
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(f.size());

    draw_sidebar(f, app, panels[0]);
    draw_main_panel(f, app, panels[1])
}
