use std::path::PathBuf;

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while opening `{}`, error: {source}", file_path.display()))]
    OpenFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while parsing `{}`, error: {source}", file_path.display()))]
    ParseFile { file_path: PathBuf, source: toml::de::Error },
}
