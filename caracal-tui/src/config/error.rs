use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },

    #[snafu(display("{source}"))]
    Profile { source: caracal_cli::profile::Error },
}

impl From<caracal_cli::profile::Error> for Error {
    fn from(source: caracal_cli::profile::Error) -> Self { Self::Profile { source } }
}
