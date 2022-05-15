use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LogLine {
    pub date: String,
    pub timestamp: String,
    pub app: String,
    pub severity: String,
    pub function: String,
    pub payload: String,
    pub color: Option<u32>,
}

impl LogLine {
   pub fn columns() -> Vec<String> {
        vec![
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
