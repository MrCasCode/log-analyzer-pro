use anyhow::{anyhow, Result};
mod models;
mod services;
mod stores;
mod domain;

use models::format::Format;

use services::log_service::{LogAnalyzer, LogService};
use services::log_source::SourceType;
use std::sync::Arc;
use std::time::Duration;
use stores::analysis_store::InMemmoryAnalysisStore;
use stores::log_store::InMemmoryLogStore;
use stores::processing_store::{ProcessingStore, InMemmoryProcessingStore};

use async_std::task;

fn get_filename() -> Option<String> {
    let file = std::env::args().skip(1).next()?;
    println!("file to stream: {:?}", file);
    return Some(file);
}

async fn async_main() -> Result<()> {
    let file = get_filename().ok_or(anyhow!("No file provided"))?;

    let log_store = Arc::new(InMemmoryLogStore::new());
    let processing_store = Arc::new(InMemmoryProcessingStore::new());
    let analysis_store = Arc::new(InMemmoryAnalysisStore::new());



    processing_store.add_format(file.clone(), r"(?P<PAYLOAD>.*)".to_string()).await;

    let mut log_service = LogService::new(log_store, processing_store, analysis_store);
    log_service.add_log(
        SourceType::FILE,
        file,
        Format::new("log".to_string(), "log".to_string()).unwrap(),
    );

    loop {
        async_std::task::sleep(Duration::from_secs(10)).await;
    }
}

fn main() -> Result<()> {
    task::block_on(async_main())?;

    Ok(())
}
