use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::log_line::LogLine;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
/// This struct contains a formated log with its info clasified
/// in several fields
pub struct LogLineStyled {
    pub log: Vec<(Option<String>, String)>,
    pub index: Vec<(Option<String>, String)>,
    pub date: Vec<(Option<String>, String)>,
    pub timestamp: Vec<(Option<String>, String)>,
    pub app: Vec<(Option<String>, String)>,
    pub severity: Vec<(Option<String>, String)>,
    pub function: Vec<(Option<String>, String)>,
    pub payload: Vec<(Option<String>, String)>,
    pub color: Option<(u8, u8, u8)>,
}

impl LogLineStyled {
    /// Returns the available fields
    pub fn columns() -> Vec<String> {
        vec![
            "Log".to_string(),
            "Index".to_string(),
            "Date".to_string(),
            "Timestamp".to_string(),
            "App".to_string(),
            "Severity".to_string(),
            "Function".to_string(),
            "Payload".to_string(),
        ]
    }

    /// Gets the field value with the `columns` returned key
    pub fn get(&self, key: &str) -> Option<&Vec<(Option<String>, String)>> {
        match key {
            "Log" => Some(&self.log),
            "Index" => Some(&self.index),
            "Date" => Some(&self.date),
            "Timestamp" => Some(&self.timestamp),
            "App" => Some(&self.app),
            "Severity" => Some(&self.severity),
            "Function" => Some(&self.function),
            "Payload" => Some(&self.payload),
            _ => None,
        }
    }

    /// Gets a (key, value) like representation of some fields
    pub fn values(&self) -> Vec<(&str, &Vec<(Option<String>, String)>)> {
        vec![
            ("Log", &self.log),
            ("Date", &self.date),
            ("Timestamp", &self.timestamp),
            ("App", &self.app),
            ("Severity", &self.severity),
            ("Function", &self.function),
            ("Payload", &self.payload),
        ]
    }


    /// Return a copy of this line with unformatted content
    pub fn unformat(&self) -> LogLine {
        let unformat = |groups: &Vec<(Option<String>, String)>| {
            groups.into_iter().fold(String::new(), |acc, g| acc + &g.1)
        };

        LogLine {
            log: unformat(&self.log),
            index: unformat(&self.index),
            date: unformat(&self.date),
            timestamp: unformat(&self.timestamp),
            app: unformat(&self.app),
            severity: unformat(&self.severity),
            function: unformat(&self.function),
            payload: unformat(&self.payload),
            color: self.color,
        }
    }
}

impl IntoIterator for LogLineStyled {
    type Item = Vec<(Option<String>, String)>;
    type IntoIter = std::array::IntoIter<Vec<(Option<String>, String)>, 7>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            self.log,
            self.date,
            self.timestamp,
            self.app,
            self.severity,
            self.function,
            self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a LogLineStyled {
    type Item = &'a Vec<(Option<String>, String)>;
    type IntoIter = std::array::IntoIter<&'a Vec<(Option<String>, String)>, 7>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.log,
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a mut LogLineStyled {
    type Item = &'a Vec<(Option<String>, String)>;
    type IntoIter = std::array::IntoIter<&'a Vec<(Option<String>, String)>, 7>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.log,
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl<'a> IntoIterator for &'a &'a mut LogLineStyled {
    type Item = &'a Vec<(Option<String>, String)>;
    type IntoIter = std::array::IntoIter<&'a Vec<(Option<String>, String)>, 7>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.log,
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}
impl<'a> IntoIterator for &'a &'a LogLineStyled {
    type Item = &'a Vec<(Option<String>, String)>;
    type IntoIter = std::array::IntoIter<&'a Vec<(Option<String>, String)>, 7>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.log,
            &self.date,
            &self.timestamp,
            &self.app,
            &self.severity,
            &self.function,
            &self.payload,
        ])
    }
}

impl Ord for LogLineStyled {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.unformat().index.parse::<usize>(), other.unformat().index.parse::<usize>()) {
            (Ok(index), Ok(other)) => match (index, other) {
                (index, other) if index < other => Ordering::Less,
                (index, other) if index == other => Ordering::Equal,
                _ => Ordering::Greater,
            },
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for LogLineStyled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.unformat().index.parse::<usize>(), other.unformat().index.parse::<usize>()) {
            (Ok(index), Ok(other)) => match (index, other) {
                (index, other) if index < other => Some(Ordering::Less),
                (index, other) if index == other => Some(Ordering::Equal),
                _ => Some(Ordering::Greater),
            },
            _ => None,
        }
    }
}

impl PartialEq for LogLineStyled {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.date == other.date
            && self.timestamp == other.timestamp
            && self.app == other.app
            && self.severity == other.severity
            && self.function == other.function
            && self.payload == other.payload
            && self.color == other.color
    }
}

impl Eq for LogLineStyled {}
