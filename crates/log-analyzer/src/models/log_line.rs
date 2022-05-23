use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LogLine {
    pub index: String,
    pub date: String,
    pub timestamp: String,
    pub app: String,
    pub severity: String,
    pub function: String,
    pub payload: String,
    pub color: Option<(u8, u8, u8)>,
}

impl LogLine {
   pub fn columns() -> Vec<String> {
        vec![
            "Index".to_string(),
            "Date".to_string(),
            "Timestamp".to_string(),
            "App".to_string(),
            "Severity".to_string(),
            "Function".to_string(),
            "Payload".to_string(),
        ]
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        match key {
            "Index" => Some(&self.index),
            "Date" => Some(&self.date),
            "Timestamp" => Some(&self.timestamp),
            "App" => Some(&self.app),
            "Severity" => Some(&self.severity),
            "Function" => Some(&self.function),
            "Payload" => Some(&self.payload),
            _ => None
        }
    }
}

impl IntoIterator for LogLine {
    type Item = String;
    type IntoIter = std::array::IntoIter<String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            self.date,
            self.timestamp,
            self.app,
            self.severity,
            self.function,
            self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a LogLine {
    type Item = &'a String;
    type IntoIter = std::array::IntoIter<&'a String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a mut LogLine {
    type Item = &'a String;
    type IntoIter = std::array::IntoIter<&'a String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a &'a mut LogLine {
    type Item = &'a String;
    type IntoIter = std::array::IntoIter<&'a String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}
impl<'a> IntoIterator for &'a &'a LogLine {
    type Item = &'a String;
    type IntoIter = std::array::IntoIter<&'a String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl Ord for LogLine {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.index.parse::<usize>(), other.index.parse::<usize>()) {
            (Ok(index), Ok(other)) => match (index, other) {
                (index, other) if index < other => Ordering::Less,
                (index, other) if index == other => Ordering::Equal,
                _ => Ordering::Greater,
            },
            _ => Ordering::Equal
        }
    }
}

impl PartialOrd for LogLine {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.index.parse::<usize>(), other.index.parse::<usize>()) {
            (Ok(index), Ok(other)) => match (index, other) {
                (index, other) if index < other => Some(Ordering::Less),
                (index, other) if index == other => Some(Ordering::Equal),
                _ => Some(Ordering::Greater),
            },
            _ => None
        }
    }
}

impl PartialEq for LogLine {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.date == other.date && self.timestamp == other.timestamp && self.app == other.app && self.severity == other.severity && self.function == other.function && self.payload == other.payload && self.color == other.color
    }
}

impl Eq for LogLine {

}