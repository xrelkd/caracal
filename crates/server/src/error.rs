use std::path::PathBuf;

use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while initializing downloader factory, error: {source}"))]
    InitializeDownloader { source: caracal_engine::Error },

    #[snafu(display("Error occurs while starting tonic server, error: {source}"))]
    StartTonicServer { source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display("Error occurs while creating Unix domain socket listener on `{}`, error: {source}", socket_path.display()))]
    CreateUnixListener { socket_path: PathBuf, source: std::io::Error, backtrace: Backtrace },

    #[snafu(display("Error occurs while starting dbus service, error: {source}"))]
    StartDBusService { source: zbus::Error },

    #[snafu(display("Error occurs while building DownloaderFactory, error: {source}"))]
    BuildDownloaderFactory { source: caracal_engine::Error },

    #[snafu(display("Error occurs while binding web server, error: {source}"))]
    BindWebServer { source: std::io::Error },

    #[snafu(display("Error occurs while serving web server, error: {source}"))]
    ServeBindWebServer { source: std::io::Error },

    #[snafu(display("{source}"))]
    Metrics { source: caracal_metrics::Error },
}

impl From<zbus::Error> for Error {
    fn from(source: zbus::Error) -> Self { Self::StartDBusService { source } }
}

impl From<caracal_metrics::Error> for Error {
    fn from(source: caracal_metrics::Error) -> Self { Self::Metrics { source } }
}
