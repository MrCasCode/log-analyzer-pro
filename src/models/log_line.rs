use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogLine {
    pub date: String,
    pub timestamp: String,
    pub app: String,
    pub severity: String,
    pub function: String,
    pub payload: String,
    pub color: Option<u32>,
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

impl<'a> IntoIterator for &'a&'a mut LogLine {
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
impl<'a> IntoIterator for &'a&'a LogLine {
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
