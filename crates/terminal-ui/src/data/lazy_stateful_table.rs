use std::sync::{Arc, RwLock};

use tui::widgets::TableState;

use super::Stateful;

const CAPACITY: usize = 1000;
const ROOM: usize = 100;

pub trait LazySource<T> {
    fn source(&self, from: usize, to: usize) -> Vec<T>;
}

enum Area {
    Below,
    Inside,
    Above,
}

impl Area {
    fn current_area(i: usize) -> Area {
        match i {
            i if i < (CAPACITY / 2 - ROOM) => Area::Below,
            i if (CAPACITY / 2 - ROOM) <= i && i <= (CAPACITY / 2 + ROOM) => Area::Inside,
            i if i > (CAPACITY / 2 + ROOM) => Area::Above,
            _ => Area::Below,
        }
    }
}

struct HackTableState {
    offset: usize,
    selected: Option<usize>,
}

pub struct LazyStatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
    offset: usize,
    source: Box<dyn LazySource<T>>,
}

impl<T> LazyStatefulTable<T> {
    pub fn new(source: Box<dyn LazySource<T>>) -> LazyStatefulTable<T> {
        let items = source.source(0, CAPACITY);
        LazyStatefulTable {
            state: TableState::default(),
            items,
            offset: 0,
            source,
        }
    }
}

impl<T> Stateful<T> for LazyStatefulTable<T> {
    fn next(&mut self) {
        if self.items.len() == 0 {
            self.items = self.source.source(0, CAPACITY)
        }
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => match Area::current_area(i) {
                    Area::Below | Area::Inside => {
                        if (i + 1) < self.items.len() {
                            i + 1
                        } else {
                            i
                        }
                    }
                    Area::Above => {
                        let last_element = CAPACITY + self.offset;

                        let new_data = self.source.source(last_element, last_element + ROOM);

                        let received_elements = new_data.len();
                        self.items.rotate_left(received_elements);

                        self.items[(CAPACITY - received_elements)..CAPACITY]
                            .iter_mut()
                            .zip(new_data)
                            .for_each(|(current, new_data)| *current = new_data);
                        self.offset += received_elements;

                        self.state.select(None);
                        i - received_elements + if (i + 1) < self.items.len() { 1 } else { 0 }
                    }
                },

                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn previous(&mut self) {
        if self.items.len() == 0 {
            self.items = self.source.source(0, CAPACITY)
        }
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => match Area::current_area(i) {
                    Area::Above | Area::Inside => {
                        if i > 0 {
                            i - 1
                        } else {
                            i
                        }
                    }
                    Area::Below => {
                        let initial_element = if self.offset > ROOM {
                            self.offset - ROOM
                        } else {
                            0
                        };

                        let new_data = self.source.source(initial_element, self.offset);

                        let received_elements = new_data.len();
                        self.items.rotate_right(received_elements);

                        self.items[0..received_elements]
                            .iter_mut()
                            .zip(new_data)
                            .for_each(|(current, new_data)| *current = new_data);
                        self.offset -= received_elements;

                        let selected = i + received_elements - if i > 0 { 1 } else { 0 };

                        unsafe {
                            self.state = std::mem::transmute::<(usize, Option<usize>), TableState>((selected, None))
                        }

                        selected
                    }
                },

                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSourcer<T> {
        items: Vec<T>,
    }

    impl<T: Clone> LazySource<T> for TestSourcer<T> {
        fn source(&self, from: usize, to: usize) -> Vec<T> {
            self.items[from.min(self.items.len())..to.min(self.items.len())].to_vec()
        }
    }

    #[test]
    fn source_init() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let lazy_table = LazyStatefulTable::new(Box::new(test_source));

        assert!(lazy_table.items.len() == CAPACITY)
    }

    #[test]
    fn single_next_doesnt_source() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let mut lazy_table = LazyStatefulTable::new(Box::new(test_source));
        lazy_table.next();
        assert!(lazy_table.items[0] == 0 && *lazy_table.items.last().unwrap() == 999);
    }

    #[test]
    fn double_next_doesnt_source() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let mut lazy_table = LazyStatefulTable::new(Box::new(test_source));
        lazy_table.next();
        lazy_table.next();
        assert!(lazy_table.items[0] == 0 && *lazy_table.items.last().unwrap() == 999);
    }

    #[test]
    fn next_inside_doesnt_source() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let mut lazy_table = LazyStatefulTable::new(Box::new(test_source));
        for _ in 0..(CAPACITY / 2) {
            lazy_table.next();
        }
        assert!(lazy_table.items[0] == 0 && *lazy_table.items.last().unwrap() == 999);
    }

    #[test]
    fn next_outside_sources() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let mut lazy_table = LazyStatefulTable::new(Box::new(test_source));
        lazy_table.state.select(Some(CAPACITY / 2 + ROOM + 1));
        lazy_table.next();
        assert!(lazy_table.items[0] == 100 && *lazy_table.items.last().unwrap() == 1099);
    }

    #[test]
    fn previous_outside_sources() {
        let test_source = TestSourcer {
            items: (0..2000_usize).collect(),
        };
        let mut lazy_table = LazyStatefulTable::new(Box::new(test_source));
        lazy_table.state.select(Some(CAPACITY / 2 + ROOM + 1));
        lazy_table.next();
        lazy_table.state.select(Some(CAPACITY / 2 - ROOM - 1));
        lazy_table.previous();
        assert!(lazy_table.items[0] == 0 && *lazy_table.items.last().unwrap() == 999);
    }
}
