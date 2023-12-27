extern crate http as hyper_http;

mod downloader;
mod error;
mod ext;
mod fetcher;
mod task_scheduler;

pub use self::{
    downloader::{Downloader, DownloaderFactory, DownloaderStatus, NewTask},
    error::Error,
    task_scheduler::TaskScheduler,
};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum Priority {
    Lowest = 0,

    Low = 1,
    #[default]
    Normal = 2,

    High = 3,

    Highest = 4,
}
