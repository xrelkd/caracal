use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DaemonConfig {
    #[serde(default = "caracal_base::config::default_server_endpoint", with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,
}

impl DaemonConfig {
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
