use super::log_line::LogLine;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum FilterAction {
    /// Just add a color marker
    MARKER,
    /// Exclude what is not matched by this filter
    INCLUDE,
    /// Exclude what is matched by this filter
    EXCLUDE
}


#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub action: FilterAction,
    pub filter: LogLine
}


