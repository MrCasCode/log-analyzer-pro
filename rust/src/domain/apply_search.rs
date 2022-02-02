use crate::models::log_line::LogLine;

pub fn apply_search(search: String, log_line: &LogLine) -> bool{
    for group in log_line {
        if let Some(_) = group.find(&search) {
            return true;
        }
    }

    false
}