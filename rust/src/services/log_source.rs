

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_std::io::prelude::{SeekExt, BufReadExt};
use async_std::{prelude::*, task, channel};
use async_std::task::JoinHandle;
use async_std::{fs::File, io::BufReader, io::Seek, io::SeekFrom, stream::Stream};
//use tokio::sync::{broadcast::Sender};
use async_std::{prelude::*, channel::Receiver, channel::Sender};
use futures::{StreamExt};

use crate::models::log::Log;

use anyhow::{Result, anyhow};

use async_trait::async_trait;



#[derive(PartialEq)]
pub enum SourceType {
    FILE,
    WS
}

pub fn create_source(source: SourceType, source_address: String) -> Box<dyn LogSource + Send + Sync> {
    if source == SourceType::FILE {
        Box::new(FileSource{path: source_address})
    }
    else {
        Box::new(WsSource{address: source_address})
    }
}

#[async_trait]
pub trait LogSource {
    async fn run(&self, sender: Sender<(String, String)>) -> Result<()>;
}



pub struct FileSource {
    path: String,
}

impl FileSource {
}

#[async_trait]
impl LogSource for FileSource {
    async fn run(&self, sender: Sender<(String, String)>) -> Result<()> {
        println!("Run {}", self.path);
        let mut read_lines = 0_usize;
        loop {
            let file = File::open(&self.path).await;
            match file {
                Ok(f) => {
                    let reader = BufReader::with_capacity(2_usize.pow(26), f);

                    let mut lines = reader.lines().skip(read_lines);
                    while let Some(line) = lines.next().await {
                        sender.send((self.path.clone(), line?)).await?;
                        read_lines += 1;
                    }
                },
                Err(err) => println!("{:?}", err)
            }

            async_std::task::sleep(Duration::from_millis(125)).await;
        }
    }
}







pub struct WsSource {
    address: String
}

#[async_trait]
impl LogSource for WsSource {
    async fn run(&self, sender: Sender<(String, String)>) -> Result<()> {
        unimplemented!()
    }
}