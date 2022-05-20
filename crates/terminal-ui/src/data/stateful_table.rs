use std::sync::{RwLock, Arc};

use tui::widgets::TableState;

use super::Stateful;

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Arc<RwLock<Vec<T>>>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Arc<RwLock<Vec<T>>>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }
}

impl<T> Stateful<T> for StatefulTable<T> {
    fn next(&mut self) {
        if self.items.read().unwrap().len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.read().unwrap().len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn previous(&mut self) {
        if self.items.read().unwrap().len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.read().unwrap().len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}
