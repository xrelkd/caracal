use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebConfig {
    #[serde(default = "WebConfig::default_enable")]
    pub enable: bool,

    #[serde(default = "WebConfig::default_host")]
    pub host: IpAddr,

    #[serde(default = "WebConfig::default_port")]
    pub port: u16,
}

impl WebConfig {
    #[inline]
    pub const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.host, self.port) }

    #[inline]
    pub const fn default_enable() -> bool { true }

    #[inline]
    pub const fn default_host() -> IpAddr { caracal_base::DEFAULT_WEB_HOST }

    #[inline]
    pub const fn default_port() -> u16 { caracal_base::DEFAULT_WEB_PORT }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enable: Self::default_enable(),
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

impl From<WebConfig> for caracal_server::config::WebConfig {
    fn from(config: WebConfig) -> Self {
        Self { enable: config.enable, listen_address: config.socket_address() }
    }
}
