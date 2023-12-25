use std::{collections::HashMap, path::PathBuf};

use snafu::{OptionExt, ResultExt};
use tokio::fs::OpenOptions;

pub use crate::error::Error;
use crate::{
    downloader::{Downloader, TransferStatus},
    error,
    ext::UrlExt,
    fetcher::Fetcher,
    minio::MinioAlias,
    ssh::SshConfig,
};

#[derive(Clone, Debug)]
pub struct Factory {
    pub http_client: reqwest::Client,

    pub default_worker_number: u64,

    pub minimum_chunk_size: u64,

    pub minio_aliases: HashMap<String, MinioAlias>,

    pub ssh_servers: HashMap<String, SshConfig>,
}

impl Default for Factory {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            default_worker_number: 5,
            minimum_chunk_size: 100 * 1024,
            minio_aliases: HashMap::new(),
            ssh_servers: HashMap::new(),
        }
    }
}

impl Factory {
    /// # Errors
    pub async fn create_new_task(&self, new_task: NewTask) -> Result<Downloader, Error> {
        let source = match new_task.url.scheme() {
            "http" | "https" => Fetcher::new_http(self.http_client.clone(), new_task.url.clone()),
            "file" => Fetcher::new_file(new_task.url.clone()).await?,
            "sftp" => {
                let endpoint = new_task.url.host_str().context(error::HostnameNotProvidedSnafu)?;
                let file_path = new_task.url.path();
                let SshConfig { endpoint, user, identity_file } = self
                    .ssh_servers
                    .get(endpoint)
                    .context(error::SshConfigNotFoundSnafu { endpoint: endpoint.to_string() })?;
                Fetcher::new_sftp(endpoint, user, identity_file, file_path).await?
            }
            "minio" => {
                let minio_path = new_task
                    .url
                    .minio_path()
                    .with_context(|| error::InvalidMinioUrlSnafu { url: new_task.url.clone() })?;

                let alias = self
                    .minio_aliases
                    .get(&minio_path.alias)
                    .context(error::MinioAliasNotFoundSnafu { alias: minio_path.alias.clone() })?;

                Fetcher::new_minio(
                    &alias.endpoint_url,
                    &alias.access_key,
                    &alias.secret_key,
                    minio_path.bucket,
                    minio_path.object,
                )?
            }
            scheme => return Err(Error::UnsupportedScheme { scheme: scheme.to_string() }),
        };

        match source.fetch_metadata().await {
            Ok(metadata) => {
                let filename = new_task.filename.unwrap_or_else(|| metadata.filename.clone());
                let full_path =
                    [&new_task.directory_path, &filename].into_iter().collect::<PathBuf>();
                let sink = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&full_path)
                    .await
                    .with_context(|_| error::CreateFileSnafu { file_path: filename.clone() })?;
                sink.set_len(metadata.length)
                    .await
                    .with_context(|_| error::ResizeFileSnafu { file_path: filename.clone() })?;

                let (chunk_size, worker_number) = if metadata.length <= self.minimum_chunk_size {
                    (metadata.length, 1)
                } else {
                    let worker_number =
                        new_task.worker_number.unwrap_or(self.default_worker_number);
                    let worker_number =
                        if worker_number == 0 { self.default_worker_number } else { worker_number };
                    (metadata.length / worker_number, worker_number)
                };
                let transfer_status = TransferStatus::new(metadata.length, chunk_size)?;

                Ok(Downloader {
                    use_simple: false,
                    worker_number,
                    transfer_status,
                    sink,
                    source,
                    url: new_task.url.clone(),
                    filename: full_path,
                    handle: None,
                })
            }
            Err(Error::NoLength) => {
                let filename = new_task.filename.unwrap_or_else(|| new_task.url.guess_filename());
                let full_path =
                    [&new_task.directory_path, &filename].into_iter().collect::<PathBuf>();
                let sink = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&full_path)
                    .await
                    .with_context(|_| error::CreateFileSnafu { file_path: filename.clone() })?;
                sink.set_len(0)
                    .await
                    .with_context(|_| error::ResizeFileSnafu { file_path: filename.clone() })?;
                let transfer_status = TransferStatus::unknown_length();
                Ok(Downloader {
                    use_simple: true,
                    worker_number: 1,
                    transfer_status,
                    sink,
                    source,
                    url: new_task.url.clone(),
                    filename: full_path,
                    handle: None,
                })
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewTask {
    pub url: reqwest::Url,

    pub filename: Option<PathBuf>,

    pub directory_path: PathBuf,

    pub worker_number: Option<u64>,
}
