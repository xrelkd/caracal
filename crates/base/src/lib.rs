pub mod config;
pub mod serde;
pub mod utils;

use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use directories::ProjectDirs;
use lazy_static::lazy_static;

pub const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONTROL_FILE_SUFFIX: &str = "caracal";

pub const DBUS_SERVICE_NAME: &str = "org.caracal.caracal-daemon";
pub const DBUS_OBJECT_PATH_PREFIX: &str = "/org/caracal/caracal-daemon";
pub const DBUS_SYSTEM_OBJECT_PATH: &str = "/org/caracal/caracal-daemon/system";
pub const DBUS_MANAGER_OBJECT_PATH: &str = "/org/caracal/caracal-daemon/manager";

pub const FALLBACK_FILENAME: &str = "index.html";

lazy_static! {
    pub static ref PROJECT_SEMVER: semver::Version = semver::Version::parse(PROJECT_VERSION)
        .unwrap_or(semver::Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY
        });
}

pub const PROJECT_NAME: &str = "caracal";
pub const PROJECT_NAME_WITH_INITIAL_CAPITAL: &str = "Caracal";

pub const NOTIFICATION_SUMMARY: &str = "Caracal - File Downloader";

pub const CLI_PROGRAM_NAME: &str = "caracal";
pub const CLI_CONFIG_NAME: &str = "caracal.toml";

pub const DAEMON_PROGRAM_NAME: &str = "caracal-daemon";
pub const DAEMON_CONFIG_NAME: &str = "caracal-daemon.toml";

pub const DEFAULT_GRPC_PORT: u16 = 37000;
pub const DEFAULT_GRPC_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub const DEFAULT_WEBUI_PORT: u16 = 37001;
pub const DEFAULT_WEBUI_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub const DEFAULT_METRICS_PORT: u16 = 37002;
pub const DEFAULT_METRICS_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

lazy_static::lazy_static! {
pub static ref PROJECT_CONFIG_DIR: PathBuf = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME)
            .expect("Creating `ProjectDirs` should always success")
            .config_dir()
            .to_path_buf();
}
