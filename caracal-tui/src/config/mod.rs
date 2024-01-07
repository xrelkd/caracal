mod daemon;
mod error;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use resolve_path::PathResolveExt as _;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub use self::{daemon::DaemonConfig, error::Error};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,

    #[serde(default)]
    pub log: caracal_cli::config::LogConfig,
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [
            caracal_base::PROJECT_CONFIG_DIR.to_path_buf(),
            PathBuf::from(caracal_base::TUI_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read_to_string(&path)
            .context(error::OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let mut config: Self = toml::from_str(&data)
            .context(error::ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        if let Some(ref file_path) = config.daemon.access_token_file_path {
            if let Ok(file_path) = file_path.try_resolve().map(Cow::into_owned) {
                if let Ok(token) = std::fs::read_to_string(file_path) {
                    config.daemon.access_token = Some(token.trim_end().to_string());
                }
            }
        }

        Ok(config)
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load(&path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!("Failed to read config file ({:?}), error: {err:?}", &path.as_ref(),);
                Self::default()
            }
        }
    }
}
