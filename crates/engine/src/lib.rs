extern crate http as hyper_http;

mod downloader;
mod error;
mod ext;
mod fetcher;
pub mod minio;
mod progress;
pub mod ssh;

pub use self::{
    downloader::{Downloader, DownloaderFactory, NewTask},
    error::Error,
    progress::Progress,
};
