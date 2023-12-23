use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub log: caracal_cli::config::LogConfig,
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [
            caracal_base::PROJECT_CONFIG_DIR.to_path_buf(),
            PathBuf::from(caracal_base::DAEMON_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read_to_string(&path)
            .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let config: Self = toml::from_str(&data)
            .context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        Ok(config)
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load(&path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!("Failed to read config file ({:?}), error: {err:?}", &path.as_ref());
                Self::default()
            }
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}
