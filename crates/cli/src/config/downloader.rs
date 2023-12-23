use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct DownloaderConfig {
    pub http: HttpConfig,
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
    pub fn default_user_agent() -> String {
        format!(
            "{}/{}",
            caracal_base::PROJECT_NAME_WITH_INITIAL_CAPITAL,
            caracal_base::PROJECT_VERSION
        )
    }

    pub const fn default_concurrent_connections() -> u16 { 5 }
}
