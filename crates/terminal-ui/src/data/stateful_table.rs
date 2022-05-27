use tui::widgets::TableState;

use super::Stateful;

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }
}

impl<T> Stateful<T> for StatefulTable<T> {
    fn next(&mut self) -> usize {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
        self.state.selected().unwrap_or_default()
    }

    fn previous(&mut self) -> usize {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
        self.state.selected().unwrap_or_default()
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}
