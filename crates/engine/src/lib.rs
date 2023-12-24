mod downloader;
mod error;
mod ext;
mod fetcher;
pub mod minio;
mod progress;

pub use self::{
    downloader::{Downloader, Factory, NewTask},
    error::Error,
    progress::Progress,
};
