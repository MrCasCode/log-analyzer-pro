use std::sync::Arc;


use async_std::{prelude::*, channel, channel::Receiver, channel::Sender};


//use tokio::sync::{broadcast, broadcast::Receiver, broadcast::Sender};

use futures::join;


use crate::domain::apply_filters::apply_filters;
use crate::domain::apply_format::apply_format;
use crate::domain::apply_search::apply_search;
use crate::models::filter::FilterAction;
use crate::models::{filter::Filter, format::Format, log::Log, log_line::LogLine};
use crate::stores::analysis_store::AnalysisStore;
use crate::stores::log_store::LogStore;
use crate::stores::processing_store::ProcessingStore;

use super::log_source::{create_source, LogSource, SourceType};

use chrono::prelude::*;
use std::str::FromStr;

pub trait LogAnalyzer {
    fn add_log(
        &mut self,
        source_type: SourceType,
        source_address: String,
        format: Format,
    ) -> Option<()>;
}

pub struct LogService {
    log_store: Arc<dyn LogStore + Sync + Send>,
    processing_store: Arc<dyn ProcessingStore + Sync + Send>,
    analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    source_channels: (Sender<(String, String)>, Receiver<(String, String)>),
    format_channels: (Sender<(String, String)>, Receiver<(String, String)>),
    filter_channels: (Sender<(String, LogLine)>, Receiver<(String, LogLine)>),
}

impl LogService {
    pub fn new(
        log_store: Arc<dyn LogStore + Sync + Send>,
        processing_store: Arc<dyn ProcessingStore + Sync + Send>,
        analysis_store: Arc<dyn AnalysisStore + Sync + Send>,
    ) -> Self {


        let source_channels = channel::unbounded();
        let format_channels = channel::unbounded();
        let filter_channels = channel::unbounded();

        let raw_line_log_store = log_store.clone();
        let raw_line_receiver = source_channels.1.clone();
        let raw_line_format_sender = format_channels.0.clone();
        async_std::task::spawn(async move {
            while let Ok((path, line)) = raw_line_receiver.recv().await {
                raw_line_log_store.add_line(&path, &line).await;
                raw_line_format_sender.send((path.clone(), line.clone())).await;
            }
        });

        let format_line_processing_store = processing_store.clone();
        let format_line_receiver = format_channels.1.clone();
        let format_line_filter_sender = filter_channels.0.clone();
        async_std::task::spawn(async move {
            while let Ok((path, line)) = format_line_receiver.recv().await {
                if let Some(format) = format_line_processing_store.get_format(&path).await {
                    if let Some(line) = apply_format(&format, &line) {
                        format_line_filter_sender.send((path.clone(), line)).await;
                    }
                }
            }
        });

        let filter_line_processing_store = processing_store.clone();
        let filter_line_analysis_store = analysis_store.clone();
        let filter_line_receiver = filter_channels.1.clone();
        async_std::task::spawn(async move {
            while let Ok((_path, log_line)) = filter_line_receiver.recv().await {
                let filters = filter_line_processing_store.get_filters().await;
                if let Some(filtered_line) = apply_filters(&filters, log_line) {
                    let search_query = filter_line_analysis_store.get_search_query().await;
                    filter_line_analysis_store.add_lines(&[&filtered_line]).await;

                    println!(" --> {:?}", filtered_line);

                    if search_query.is_some() && apply_search(search_query.unwrap(), &filtered_line) {
                        filter_line_analysis_store
                            .add_search_lines(&[&filtered_line])
                            .await;
                    }
                }
            }
        });

        Self {
            log_store,
            processing_store,
            analysis_store,
            source_channels,
            format_channels,
            filter_channels,
        }
    }
}

impl LogAnalyzer for LogService {
    fn add_log(
        &mut self,
        source_type: SourceType,
        source_address: String,
        format: Format,
    ) -> Option<()> {
        let sender = self.source_channels.0.clone();
        let log_store = self.log_store.clone();

        //std::thread::spawn(move ||
            {
            async_std::task::spawn(async move {
                let log_source = Arc::new(create_source(source_type, source_address.clone()));
                log_store
                    .add_log(&source_address, log_source.clone(), true)
                    .await;

                log_source.run(sender).await.unwrap();
            });
        }
    //);



        Some(())
    }
}
