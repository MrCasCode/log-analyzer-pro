
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Result};

use async_std::net::TcpStream;
use async_std::{
    fs::File,
    io::{prelude::BufReadExt, BufReader},
    prelude::StreamExt,
};
use async_trait::async_trait;
use flume::Sender;
use parking_lot::RwLock;


#[derive(Eq, PartialEq)]
pub enum SourceType {
    FILE,
    WS,
}

impl TryFrom<usize> for SourceType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SourceType::FILE),
            1 => Ok(SourceType::WS),
            _ => Err(()),
        }
    }
}

impl From<SourceType> for usize {
    fn from(val: SourceType) -> Self {
        match val {
            SourceType::FILE => 0,
            SourceType::WS => 1,
        }
    }
}

async fn is_file_path_valid(path: &String) -> bool {
    File::open(&path).await.is_ok()
}

pub async fn create_source(
    source: SourceType,
    source_address: String,
) -> Result<Box<dyn LogSource + Send + Sync>> {
    match source {
        SourceType::FILE => match is_file_path_valid(&source_address).await {
            true => Ok(Box::new(FileSource {
                path: source_address,
                read_lines: RwLock::new(0),
                enabled: AtomicBool::new(true)
            })),
            false => Err(anyhow!(
                "Could not open file.\nPlease ensure that path is correct"
            )),
        },
        SourceType::WS => Ok(Box::new(WsSource {
            address: source_address,
            enabled: AtomicBool::new(true)
        })),
    }
}

#[async_trait]
pub trait LogSource {
    async fn run(&self, sender: Sender<(String, Vec<String>)>) -> Result<()>;
    fn stop(&self);
    fn get_address(&self) -> String;
}

pub struct FileSource {
    path: String,
    read_lines: RwLock<usize>,
    enabled: AtomicBool
}

#[async_trait]
impl LogSource for FileSource {
    async fn run(&self, sender: Sender<(String, Vec<String>)>) -> Result<()> {
        let capacity = 1_000_000_usize;
        while self.enabled.load(Ordering::Relaxed) {
            let file = File::open(&self.path).await;
            match file {
                Ok(f) => {
                    let reader = BufReader::with_capacity(2_usize.pow(26), f);
                    let mut v = Vec::with_capacity(capacity);
                    let mut lines = reader.lines().skip(*self.read_lines.read());
                    while let Some(line) = lines.next().await {
                        v.push(line?);
                        if v.len() >= capacity - 1 {
                            sender.send_async((self.path.clone(), v)).await?;
                            v = Vec::with_capacity(capacity);
                        }
                        *self.read_lines.write() += 1;
                    }
                    sender.send((self.path.clone(), v))?;
                }
                Err(_) => break,
            }

            async_std::task::sleep(Duration::from_millis(300)).await;
        }
        // restore after quitting
        self.enabled.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    fn get_address(&self) -> String {
        self.path.clone()
    }

}

pub struct WsSource {
    address: String,
    enabled: AtomicBool
}

#[async_trait]
impl LogSource for WsSource {
    async fn run(&self, sender: Sender<(String, Vec<String>)>) -> Result<()> {
        while self.enabled.load(Ordering::Relaxed) {
            let stream = match TcpStream::connect(&self.address).await {
                Ok(stream) => Some(stream),
                Err(_) => None,
            };
            if let Some(stream) = stream {
                while self.enabled.load(Ordering::Relaxed) {
                    let mut lines_from_server = BufReader::new(&stream).lines().fuse();
                    match lines_from_server.next().await {
                        Some(line) => {
                            let line = line?;
                            sender.send((self.address.clone(), vec![line]))?;
                        }
                        None => break,
                    }
                }
            }
            async_std::task::sleep(Duration::from_secs(3)).await;
        }
        // restore after quitting
        self.enabled.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    fn get_address(&self) -> String {
        self.address.clone()
    }
}
