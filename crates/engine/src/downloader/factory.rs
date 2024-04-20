use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use caracal_base::{
    model,
    profile::{minio::MinioAlias, ssh::SshConfig},
};
use futures::{future, FutureExt};
use snafu::{OptionExt, ResultExt};
use tokio::fs::OpenOptions;

pub use crate::error::Error;
use crate::{
    downloader::{control_file::ControlFile, Downloader, TransferStatus},
    error,
    ext::UriExt,
    fetcher::Fetcher,
};

#[derive(Clone, Debug)]
pub struct Builder {
    pub default_concurrent_number: u64,

    pub default_output_directory_path: PathBuf,

    pub http_user_agent: Option<String>,

    pub minimum_chunk_size: u64,

    pub minio_aliases: HashMap<String, MinioAlias>,

    pub ssh_servers: HashMap<String, SshConfig>,

    pub connection_timeout: Duration,
}

impl Builder {
    /// # Errors
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            default_concurrent_number: 5,
            default_output_directory_path: std::env::current_dir()
                .context(error::GetCurrentDirectorySnafu)?,
            http_user_agent: None,
            minimum_chunk_size: 100 * 1024,
            minio_aliases: HashMap::new(),
            ssh_servers: HashMap::new(),
            connection_timeout: Duration::from_secs(60),
        })
    }

    pub const fn default_concurrent_number(mut self, default_concurrent_number: u64) -> Self {
        self.default_concurrent_number = default_concurrent_number;
        self
    }

    pub fn default_output_directory_path<P>(mut self, default_output_directory_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.default_output_directory_path = default_output_directory_path.as_ref().to_path_buf();
        self
    }

    pub const fn minimum_chunk_size(mut self, minimum_chunk_size: u64) -> Self {
        self.minimum_chunk_size = minimum_chunk_size;
        self
    }

    pub fn minio_aliases(mut self, minio_aliases: HashMap<String, MinioAlias>) -> Self {
        self.minio_aliases = minio_aliases;
        self
    }

    pub const fn connection_timeout(mut self, connection_timeout: Duration) -> Self {
        self.connection_timeout = connection_timeout;
        self
    }

    pub fn ssh_servers(mut self, ssh_servers: HashMap<String, SshConfig>) -> Self {
        self.ssh_servers = ssh_servers;
        self
    }

    pub fn http_user_agent<S>(mut self, user_agent: S) -> Self
    where
        S: fmt::Display,
    {
        self.http_user_agent = Some(user_agent.to_string());
        self
    }

    pub fn build(self) -> Result<Factory, Error> {
        let Self {
            http_user_agent,
            default_output_directory_path,
            default_concurrent_number,
            minio_aliases,
            ssh_servers,
            minimum_chunk_size,
            connection_timeout,
        } = self;

        let http_client = reqwest::Client::builder()
            .user_agent(
                http_user_agent
                    .unwrap_or_else(|| caracal_base::DEFAULT_HTTP_USER_AGENT.to_string()),
            )
            .build()
            .context(error::BuildHttpClientSnafu)?;

        Ok(Factory {
            http_client,
            default_output_directory_path,
            default_concurrent_number,
            minimum_chunk_size,
            minio_aliases,
            ssh_servers,
            connection_timeout,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Factory {
    http_client: reqwest::Client,

    default_output_directory_path: PathBuf,

    default_concurrent_number: u64,

    minimum_chunk_size: u64,

    minio_aliases: HashMap<String, MinioAlias>,

    ssh_servers: HashMap<String, SshConfig>,

    connection_timeout: Duration,
}

impl Factory {
    /// # Errors
    #[inline]
    pub fn builder() -> Result<Builder, Error> { Builder::new() }

    /// # Errors
    pub async fn create_new_task(&self, new_task: &model::CreateTask) -> Result<Downloader, Error> {
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
        if source.supports_range_request() {
            let filename = new_task.filename.clone().unwrap_or_else(|| metadata.filename.clone());
            let full_path = [
                new_task.output_directory.as_ref().unwrap_or(&self.default_output_directory_path),
                &filename,
            ]
            .into_iter()
            .collect::<PathBuf>();
            if tokio::fs::try_exists(&full_path).await.unwrap_or(false)
                && !tokio::fs::try_exists(ControlFile::file_path(&full_path)).await.unwrap_or(false)
            {
                return Err(Error::DestinationFileExists { file_path: full_path.clone() });
            }

            let sink = OpenOptions::new()
                .create(true)
                .truncate(false)
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
                let concurrent_number =
                    new_task.concurrent_number.unwrap_or(self.default_concurrent_number);
                let concurrent_number = if concurrent_number == 0 {
                    self.default_concurrent_number
                } else {
                    concurrent_number
                };
                (metadata.length / concurrent_number, concurrent_number)
            };

            let transfer_status = TransferStatus::new(metadata.length, chunk_size)?;

            Ok(Downloader {
                use_single_worker: false,
                worker_number,
                transfer_status,
                sink,
                source,
                uri: new_task.uri.clone(),
                file_path: full_path,
                handle: None,
                is_completed: Arc::new(AtomicBool::new(false)),
            })
        } else {
            let filename =
                new_task.filename.clone().unwrap_or_else(|| new_task.uri.guess_filename());
            let full_path = [
                new_task.output_directory.as_ref().unwrap_or(&self.default_output_directory_path),
                &filename,
            ]
            .into_iter()
            .collect::<PathBuf>();
            if tokio::fs::try_exists(&full_path).await.unwrap_or(false) {
                return Err(Error::DestinationFileExists { file_path: full_path.clone() });
            }

            let sink = OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(&full_path)
                .await
                .with_context(|_| error::CreateFileSnafu { file_path: filename.clone() })?;
            sink.set_len(0)
                .await
                .with_context(|_| error::ResizeFileSnafu { file_path: filename.clone() })?;

            let transfer_status = TransferStatus::unknown_length();

            Ok(Downloader {
                use_single_worker: true,
                worker_number: 1,
                transfer_status,
                sink,
                source,
                uri: new_task.uri.clone(),
                file_path: full_path,
                handle: None,
                is_completed: Arc::new(AtomicBool::new(false)),
            })
        }
    }

    async fn create_fetcher(&self, new_task: &model::CreateTask) -> Result<Fetcher, Error> {
        match new_task.uri.scheme_str() {
            Some("file") | None => Fetcher::new_file(new_task.uri.path()).await,
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
