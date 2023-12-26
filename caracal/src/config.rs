use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use resolve_path::PathResolveExt as _;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,

    #[serde(default)]
    pub log: caracal_cli::config::LogConfig,

    #[serde(default)]
    pub downloader: caracal_cli::config::DownloaderConfig,

    #[serde(default)]
    pub profile_files: Vec<PathBuf>,
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [
            caracal_base::PROJECT_CONFIG_DIR.to_path_buf(),
            PathBuf::from(caracal_base::CLI_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read_to_string(&path)
            .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let mut config: Self = toml::from_str(&data)
            .context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        if let Some(ref file_path) = config.daemon.access_token_file_path {
            if let Ok(token) = std::fs::read_to_string(file_path) {
                config.daemon.access_token = Some(token.trim_end().to_string());
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

    pub fn profile_files(&self) -> Vec<PathBuf> {
        self.profile_files
            .iter()
            .filter_map(|path| path.try_resolve().map(Cow::into_owned).ok())
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DaemonConfig {
    #[serde(default = "caracal_base::config::default_server_endpoint", with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,
}

impl DaemonConfig {
    // FIXME: use it
    #[allow(dead_code)]
    pub fn access_token(&self) -> Option<String> { self.access_token.clone() }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            server_endpoint: caracal_base::config::default_server_endpoint(),
            access_token: None,
            access_token_file_path: None,
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
