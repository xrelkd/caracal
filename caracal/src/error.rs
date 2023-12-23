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

    #[snafu(display("Error occurs while downloading {url}, error: {error}"))]
    Downloader { url: Box<reqwest::Url>, error: caracal_engine::Error },

    #[snafu(display("Error occurs while running lifecycle manager, error: {source}"))]
    LifecycleManager { source: sigfinn::Error },

    #[snafu(display("No URL is provided"))]
    NoUrl,
}
