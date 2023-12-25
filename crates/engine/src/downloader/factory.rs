use std::{collections::HashMap, path::PathBuf, time::Duration};

use futures::{future, FutureExt};
use snafu::{OptionExt, ResultExt};
use tokio::fs::OpenOptions;

use super::control_file::ControlFile;
pub use crate::error::Error;
use crate::{
    downloader::{Downloader, TransferStatus},
    error,
    ext::UriExt,
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

    pub connection_timeout: Duration,
}

impl Default for Factory {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            default_worker_number: 5,
            minimum_chunk_size: 100 * 1024,
            minio_aliases: HashMap::new(),
            ssh_servers: HashMap::new(),
            connection_timeout: Duration::from_secs(60),
        }
    }
}

impl Factory {
    /// # Errors
    pub async fn create_new_task(&self, new_task: &NewTask) -> Result<Downloader, Error> {
        let source = {
            let source_fut = self.create_fetcher(new_task).boxed();
            let timeout =
                tokio::time::sleep(new_task.connection_timeout.unwrap_or(self.connection_timeout));

            tokio::pin!(timeout);

            match future::select(source_fut, timeout).await {
                future::Either::Left((source, _)) => source?,
                future::Either::Right((_timeout, _)) => return Err(Error::ConnectionTimedOut),
            }
        };

        let metadata = source.fetch_metadata();
        if metadata.length == 0 {
            let filename =
                new_task.filename.clone().unwrap_or_else(|| new_task.uri.guess_filename());
            let full_path = [&new_task.directory_path, &filename].into_iter().collect::<PathBuf>();
            if tokio::fs::try_exists(&full_path).await.unwrap_or(false) {
                return Err(Error::DestinationFileExists { file_path: full_path.clone() });
            }
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
                uri: new_task.uri.clone(),
                filename: full_path,
                handle: None,
            })
        } else {
            let filename = new_task.filename.clone().unwrap_or_else(|| metadata.filename.clone());
            let full_path = [&new_task.directory_path, &filename].into_iter().collect::<PathBuf>();
            if tokio::fs::try_exists(&full_path).await.unwrap_or(false)
                && !tokio::fs::try_exists(ControlFile::file_path(&full_path)).await.unwrap_or(false)
            {
                return Err(Error::DestinationFileExists { file_path: full_path.clone() });
            }

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
                let worker_number = new_task.worker_number.unwrap_or(self.default_worker_number);
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
                uri: new_task.uri.clone(),
                filename: full_path,
                handle: None,
            })
        }
    }

    async fn create_fetcher(&self, new_task: &NewTask) -> Result<Fetcher, Error> {
        match new_task.uri.scheme_str() {
            Some("file") | None => Fetcher::new_file(new_task.uri.clone()).await,
            Some("http" | "https") => {
                Fetcher::new_http(self.http_client.clone(), new_task.uri.clone()).await
            }
            Some("sftp") => {
                let endpoint = new_task.uri.host().context(error::HostnameNotProvidedSnafu)?;
                let file_path = new_task.uri.path();
                let SshConfig { endpoint, user, identity_file } = self
                    .ssh_servers
                    .get(endpoint)
                    .context(error::SshConfigNotFoundSnafu { endpoint: endpoint.to_string() })?;
                Fetcher::new_sftp(endpoint, user, identity_file, file_path).await
            }
            Some("minio") => {
                let minio_path = new_task
                    .uri
                    .minio_path()
                    .with_context(|| error::InvalidMinioUrlSnafu { uri: new_task.uri.clone() })?;

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
                )
                .await
            }
            Some(scheme) => Err(Error::UnsupportedScheme { scheme: scheme.to_string() }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewTask {
    pub uri: http::Uri,

    pub filename: Option<PathBuf>,

    pub directory_path: PathBuf,

    pub worker_number: Option<u64>,

    pub connection_timeout: Option<Duration>,
}
