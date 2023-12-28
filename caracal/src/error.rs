use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not create tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not get current directory, error: {source}"))]
    GetCurrentDirectory { source: std::io::Error },

    #[snafu(display("Output directory path {} does not exist", output_directory.display()))]
    OutputDirectoryNotExists { output_directory: PathBuf },

    #[snafu(display("Output directory path {} is a file", output_directory.display()))]
    OutputDirectoryPathIsFile { output_directory: PathBuf },

    #[snafu(display("Error occurs while interacting with server, error: {error}"))]
    Operation { error: String },

    #[snafu(display("Error occurs while initializing downloader factory, error: {source}"))]
    InitializeDownloader { source: caracal_engine::Error },

    #[snafu(display("Error occurs while downloading {uri}, error: {error}"))]
    Downloader { uri: Box<http::Uri>, error: caracal_engine::Error },

    #[snafu(display("Error occurs while running lifecycle manager, error: {source}"))]
    LifecycleManager { source: sigfinn::Error },

    #[snafu(display("{source}"))]
    Config {
        #[snafu(source(from(crate::config::Error, Box::new)))]
        source: Box<crate::config::Error>,
    },

    #[snafu(display("{source}"))]
    Client { source: caracal_grpc_client::Error },

    #[snafu(display("No URI is provided"))]
    NoUri,
}

impl From<crate::config::Error> for Error {
    fn from(source: crate::config::Error) -> Self { Self::Config { source: Box::new(source) } }
}

impl From<caracal_grpc_client::Error> for Error {
    fn from(source: caracal_grpc_client::Error) -> Self { Self::Client { source } }
}

impl From<caracal_grpc_client::error::AddUriError> for Error {
    fn from(error: caracal_grpc_client::error::AddUriError) -> Self {
        Self::Operation { error: error.to_string() }
    }
}

impl From<caracal_grpc_client::error::PauseTaskError> for Error {
    fn from(error: caracal_grpc_client::error::PauseTaskError) -> Self {
        Self::Operation { error: error.to_string() }
    }
}

impl From<caracal_grpc_client::error::ResumeTaskError> for Error {
    fn from(error: caracal_grpc_client::error::ResumeTaskError) -> Self {
        Self::Operation { error: error.to_string() }
    }
}

impl From<caracal_grpc_client::error::RemoveTaskError> for Error {
    fn from(error: caracal_grpc_client::error::RemoveTaskError) -> Self {
        Self::Operation { error: error.to_string() }
    }
}
