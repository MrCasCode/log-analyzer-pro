use std::sync::mpsc::SyncSender;

use anyhow::{anyhow, Result};

use async_std::{fs::File, io::{BufReader, prelude::BufReadExt}, prelude::StreamExt};
use async_trait::async_trait;

#[derive(PartialEq)]
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

impl Into<usize> for SourceType {
    fn into(self) -> usize {
        match self {
            SourceType::FILE => 0,
            SourceType::WS => 1,
        }
    }
}

async fn is_file_path_valid(path: &String) -> bool {
    match File::open(&path).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn create_source(
    source: SourceType,
    source_address: String,
) -> Result<Box<dyn LogSource + Send + Sync>> {
    match source {
        SourceType::FILE => match is_file_path_valid(&source_address).await {
            true => Ok(Box::new(FileSource {
                path: source_address,
            })),
            false => Err(anyhow!("Could not open file.\nPlease that path is correct")),
        },
        SourceType::WS => Ok(Box::new(WsSource {
            _address: source_address,
        })),
    }
}

#[async_trait]
pub trait LogSource {
    async fn run(&self, sender: SyncSender<(String, Vec<String>)>) -> Result<()>;
}

pub struct FileSource {
    path: String,
}

#[async_trait]
impl LogSource for FileSource {
    async fn run(&self, sender: SyncSender<(String, Vec<String>)>) -> Result<()> {
        let mut read_lines = 0_usize;
        let capacity = 1_000_000_usize;
        loop {
            let file = File::open(&self.path).await;
            match file {
                Ok(f) => {
                    let reader = BufReader::with_capacity(2_usize.pow(26), f);
                    let mut v = Vec::with_capacity(capacity);
                    let mut lines = reader.lines().skip(read_lines);
                    while let Some(line) = lines.next().await {
                        v.push(line?);
                        if v.len() == capacity - 1 {
                            sender.send((self.path.clone(), v))?;
                            v = Vec::with_capacity(capacity);
                        }
                        read_lines += 1;
                    }
                    sender.send((self.path.clone(), v))?;
                }
                Err(_) => break,
            }
        }

        Ok(())
    }
}

pub struct WsSource {
    _address: String,
}

#[async_trait]
impl LogSource for WsSource {
    async fn run(&self, _sender: SyncSender<(String, Vec<String>)>) -> Result<()> {
        unimplemented!()
    }
}
