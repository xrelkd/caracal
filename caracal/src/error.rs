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
    Profile { source: caracal_cli::profile::Error },

    #[snafu(display("No URI is provided"))]
    NoUri,
}

impl From<caracal_cli::profile::Error> for Error {
    fn from(source: caracal_cli::profile::Error) -> Self { Self::Profile { source } }
}
