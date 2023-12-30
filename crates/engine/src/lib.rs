extern crate http as hyper_http;

mod downloader;
mod error;
mod ext;
mod fetcher;
mod task_scheduler;

pub use self::{
    downloader::{Downloader, DownloaderFactory, DownloaderStatus, MINIMUM_CHUNK_SIZE},
    error::Error,
    task_scheduler::TaskScheduler,
};
