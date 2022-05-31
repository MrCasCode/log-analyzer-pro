use tui::widgets::TableState;

use super::Stateful;

pub const CAPACITY: usize = 1000;
const ROOM: usize = 100;

pub trait LazySource<T> {
    fn source(&self, from: usize, to: usize) -> Vec<T>;
    fn source_elements_containing(&self, element: T, quantity: usize) -> (Vec<T>, usize, usize);
}

enum Area {
    Below,
    Inside,
    Above,
}

impl Area {
    fn current_area(i: usize, elements: usize) -> Area {
        match i {
            i if i < ((elements / 2).overflowing_sub(ROOM).0) => Area::Below,
            i if (((elements / 2).overflowing_sub(ROOM).0)..=(elements / 2 + ROOM)).contains(&i) => Area::Inside,
            i if i > (elements / 2 + ROOM) => Area::Above,
            _ => Area::Below,
        }
    }
}

pub struct LazyStatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
    offset: usize,
    source: Box<dyn LazySource<T>>,
}

impl<T: Clone> LazyStatefulTable<T> {
    pub fn new(source: Box<dyn LazySource<T>>) -> LazyStatefulTable<T> {
        let items = source.source(0, CAPACITY);
        LazyStatefulTable {
            state: TableState::default(),
            items,
            offset: 0,
            source,
        }
    }

    pub fn reload(&mut self) {
        self.items = self.source.source(self.offset, CAPACITY);

        self.state.select(match self.state.selected() {
            Some(i) => Some(i.min(if !self.items.is_empty() {self.items.len() - 1} else {0})),
            _ => None,
        });
    }


    pub fn navigate_to(&mut self, element: T) {
        let source = self.source.source_elements_containing(element, CAPACITY);

        self.items = source.0;
        self.offset = source.1;
        self.state.select(Some(source.2));

    }


    pub fn navigate_to_bottom(&mut self) {
        let mut current = self.next();
        let mut next = self.next();
        while current != next {
            current = next;
            next = self.next();
        }
    }

    pub fn get_selected_item(&self) -> Option<T>{
        match self.state.selected() {
            Some(i) => self.items.get(i).cloned(),
            None => None
        }
    }

    pub fn clear(&mut self) {
        self.state.select(None);
        self.items.clear();
    }

    fn select_and_set_scroll_on_top(&mut self, index: usize) {
        // Need to manually set private field offset when scrolling up for smooth experience
        // Requested to make this public https://github.com/fdehau/tui-rs/issues/626
        // but using unsafe in the meantime
        unsafe {
            self.state = std::mem::transmute::<(usize, Option<usize>), TableState>((index, None))
        }
    }

}

impl<T: Clone> Stateful<T> for LazyStatefulTable<T> {
    fn next(&mut self) -> usize {
        if self.items.is_empty() {
            self.items = self.source.source(0, CAPACITY)
        }
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => match Area::current_area(i, self.items.len()) {
                    Area::Below | Area::Inside => {
                        if (i + 1) < self.items.len() {
                            i + 1
                        } else {
                            i
                        }
                    }
                    Area::Above => {
                        let len = self.items.len();
                        let last_element = len + self.offset;

                        let new_data = self.source.source(last_element, last_element + ROOM);

                        let received_elements = new_data.len();
                        self.items.rotate_left(received_elements);

                        self.items[(len - received_elements)..len]
                            .iter_mut()
                            .zip(new_data)
                            .for_each(|(current, new_data)| *current = new_data);
                        self.offset += received_elements;

                        self.state.select(None);
                        i - received_elements + if (i + 1) < len { 1 } else { 0 }
                    }
                },

                None => 0,
            };
            self.state.select(Some(i));
        }

        self.state.selected().unwrap_or_default()
    }

    fn previous(&mut self) -> usize {
        if self.items.is_empty() {
            self.items = self.source.source(0, CAPACITY)
        }
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => match Area::current_area(i, self.items.len()) {
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

                        let selected = i + received_elements - if i > 0 { 1 } else { 0 };

                        if received_elements > 0 {
                            self.items.rotate_right(received_elements);

                            self.items[0..received_elements]
                                .iter_mut()
                                .zip(new_data)
                                .for_each(|(current, new_data)| *current = new_data);
                            self.offset -= received_elements;


                            self.select_and_set_scroll_on_top(selected);


                        }
                        selected
                    }
                },

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

        fn source_elements_containing(
            &self,
            element: T,
            quantity: usize,
        ) -> (Vec<T>, usize, usize) {
            todo!()
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
