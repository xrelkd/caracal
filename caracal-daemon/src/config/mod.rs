mod dbus;
mod error;
mod grpc;
mod mertrics;
mod task_scheduler;
mod web;

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use caracal_base::profile::{minio::MinioAlias, ssh::SshConfig};
use caracal_cli::profile::{self, Profile, ProfileItem};
use resolve_path::PathResolveExt as _;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub use self::{
    dbus::DBusConfig, error::Error, grpc::GrpcConfig, mertrics::MetricsConfig,
    task_scheduler::TaskSchedulerConfig, web::WebConfig,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub log: caracal_cli::config::LogConfig,

    #[serde(default)]
    pub task_scheduler: TaskSchedulerConfig,

    #[serde(default)]
    pub downloader: caracal_cli::config::DownloaderConfig,

    #[serde(default)]
    pub profile_files: Vec<PathBuf>,

    #[serde(default)]
    pub grpc: GrpcConfig,

    #[serde(default)]
    pub dbus: DBusConfig,

    #[serde(default)]
    pub web: WebConfig,

    #[serde(default)]
    pub metrics: MetricsConfig,
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
            .context(error::OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let mut config: Self = toml::from_str(&data)
            .context(error::ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        if let Some(ref file_path) = config.grpc.access_token_file_path {
            if let Ok(file_path) = file_path.try_resolve().map(Cow::into_owned) {
                if let Ok(token) = std::fs::read_to_string(file_path) {
                    config.grpc.access_token = Some(token.trim_end().to_string());
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
                tracing::warn!("Failed to read config file ({:?}), error: {err:?}", &path.as_ref());
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

    pub async fn into_server_config(self) -> Result<caracal_server::Config, Error> {
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

        let grpc_listen_address = self.grpc.enable_http.then_some(self.grpc.socket_address());
        let grpc_local_socket = self.grpc.enable_local_socket.then_some(self.grpc.local_socket);
        let grpc_access_token = if let Some(file_path) = self.grpc.access_token_file_path {
            if let Ok(token) = std::fs::read_to_string(file_path) {
                Some(token.trim_end().to_string())
            } else {
                self.grpc.access_token
            }
        } else {
            self.grpc.access_token
        };

        let dbus = caracal_server::config::DBusConfig::from(self.dbus);
        let metrics = caracal_server::config::MetricsConfig::from(self.metrics);
        let web = caracal_server::config::WebConfig::from(self.web);
        let task_scheduler = caracal_server::config::TaskSchedulerConfig {
            http: caracal_server::config::HttpConfig {
                user_agent: self.downloader.http.user_agent,
                concurrent_connections: self.downloader.http.concurrent_connections,
            },
            concurrent_number: self.task_scheduler.concurrent_number,
            default_output_directory: self
                .downloader
                .default_output_directory
                .unwrap_or(std::env::current_dir().context(error::GetCurrentDirectorySnafu)?),
        };

        Ok(caracal_server::Config {
            task_scheduler,
            ssh_servers,
            minio_aliases,
            grpc_listen_address,
            grpc_local_socket,
            grpc_access_token,
            dbus,
            web,
            metrics,
        })
    }
}
