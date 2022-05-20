pub mod stateful_list;
pub mod stateful_table;
pub mod lazy_stateful_table;

pub trait Stateful<T> {
    fn next(&mut self) -> usize;
    fn previous(&mut self) -> usize;
    fn unselect(&mut self);
}