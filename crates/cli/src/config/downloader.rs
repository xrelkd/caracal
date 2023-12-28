use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct DownloaderConfig {
    pub http: HttpConfig,

    pub default_output_directory: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpConfig {
    #[serde(default = "HttpConfig::default_user_agent")]
    pub user_agent: String,

    #[serde(default = "HttpConfig::default_concurrent_connections")]
    pub concurrent_connections: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            user_agent: Self::default_user_agent(),
            concurrent_connections: Self::default_concurrent_connections(),
        }
    }
}

impl HttpConfig {
    pub fn default_user_agent() -> String { caracal_base::DEFAULT_HTTP_USER_AGENT.to_string() }

    pub const fn default_concurrent_connections() -> u16 { 5 }
}
