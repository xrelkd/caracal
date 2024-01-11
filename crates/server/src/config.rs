use std::{collections::HashMap, net::SocketAddr, path::PathBuf, time::Duration};

use caracal_base::profile::{minio::MinioAlias, ssh::SshConfig};

#[derive(Clone, Debug)]
pub struct Config {
    pub task_scheduler: TaskSchedulerConfig,

    pub ssh_servers: HashMap<String, SshConfig>,

    pub minio_aliases: HashMap<String, MinioAlias>,

    pub grpc_listen_address: Option<SocketAddr>,

    pub grpc_local_socket: Option<PathBuf>,

    pub grpc_access_token: Option<String>,

    pub dbus: DBusConfig,

    pub web: WebConfig,

    pub metrics: MetricsConfig,
}

#[derive(Clone, Debug)]
pub struct TaskSchedulerConfig {
    pub http: HttpConfig,

    pub concurrent_number: usize,

    pub default_output_directory: PathBuf,
}

#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub user_agent: String,

    pub concurrent_connections: u16,
}

#[derive(Clone, Debug)]
pub struct DBusConfig {
    pub enable: bool,

    pub identifier: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub enable: bool,

    pub listen_address: SocketAddr,
}

#[derive(Clone, Debug)]
pub struct MetricsConfig {
    pub enable: bool,

    pub listen_address: SocketAddr,
}

#[derive(Clone, Debug)]
pub struct DesktopNotificationConfig {
    pub enable: bool,

    pub icon: PathBuf,

    pub timeout: Duration,

    pub long_plaintext_length: usize,
}
