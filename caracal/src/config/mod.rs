mod daemon;
mod error;

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use caracal_base::profile::{minio::MinioAlias, ssh::SshConfig};
use caracal_cli::{
    profile,
    profile::{Profile, ProfileItem},
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
            .context(error::OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let mut config: Self = toml::from_str(&data)
            .context(error::ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

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

    pub async fn load_profiles(&self) -> Result<Profiles, Error> {
        let mut minio_aliases = HashMap::new();
        let mut ssh_servers = HashMap::new();
        for profile_file in self.profile_files() {
            for profile_item in Profile::load(profile_file).await?.profiles {
                match profile_item {
                    ProfileItem::Ssh(profile::Ssh { name, endpoint, user, identity_file }) => {
                        let config = SshConfig {
                            endpoint,
                            user,
                            identity_file: identity_file.to_string_lossy().to_string(),
                        };
                        drop(ssh_servers.insert(name, config));
                    }
                    ProfileItem::Minio(profile::Minio {
                        name,
                        endpoint_url,
                        access_key,
                        secret_key,
                    }) => {
                        let alias = MinioAlias { endpoint_url, access_key, secret_key };
                        drop(minio_aliases.insert(name, alias));
                    }
                }
            }
        }

        Ok(Profiles { minio_aliases, ssh_servers })
    }
}

#[derive(Clone, Debug)]
pub struct Profiles {
    pub minio_aliases: HashMap<String, MinioAlias>,
    pub ssh_servers: HashMap<String, SshConfig>,
}
