mod error;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub use self::error::Error;
use self::error::Result;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub profiles: Vec<ProfileItem>,
}

impl Profile {
    /// # Errors
    pub async fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Send + Sync,
    {
        let contents = tokio::fs::read_to_string(&path)
            .await
            .context(error::OpenFileSnafu { file_path: path.as_ref().to_path_buf() })?;

        toml::from_str(&contents)
            .context(error::ParseFileSnafu { file_path: path.as_ref().to_path_buf() })
    }

    /// # Errors
    pub fn load_blocking<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let contents = std::fs::read_to_string(&path)
            .context(error::OpenFileSnafu { file_path: path.as_ref().to_path_buf() })?;

        toml::from_str(&contents)
            .context(error::ParseFileSnafu { file_path: path.as_ref().to_path_buf() })
    }

    #[must_use]
    pub fn example() -> Self {
        let minio = ProfileItem::Minio(Minio {
            name: "minio-example".to_string(),
            endpoint_url: http::Uri::from_static("https://play.min.io"),
            access_key: "access_key".to_string(),
            secret_key: "secret_key".to_string(),
        });

        let ssh = ProfileItem::Ssh(Ssh {
            name: "ssh-example".to_string(),
            endpoint: "www.example.com".to_string(),
            user: "test".to_string(),
            identity_file: PathBuf::from("/path/to/identity/file"),
        });

        Self { profiles: vec![minio, ssh] }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfileItem {
    Minio(Minio),
    Ssh(Ssh),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Minio {
    pub name: String,

    #[serde(with = "caracal_base::serde::uri")]
    pub endpoint_url: http::Uri,

    pub access_key: String,

    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ssh {
    pub name: String,

    pub endpoint: String,

    pub user: String,

    pub identity_file: PathBuf,
}
