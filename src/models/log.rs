use async_std::sync::RwLock;
use async_std::channel::Receiver;

use super::format::Format;

pub struct Log {
    pub format: Format,
    pub lines: RwLock<Vec<String>>
}


impl Log {
    pub fn new(format: Format) -> Self {
        Log {format: format, lines: RwLock::new(Vec::new())}
    }

    pub async fn add_line(&self, line: String) -> String {
        let mut v = self.lines.write().await;
        println!("{}", line);
        v.push(line);
        v.last().unwrap().clone()
    }
}