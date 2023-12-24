use std::{fmt, path::PathBuf};

use opendal::{services, Operator};
use snafu::ResultExt;

use crate::{
    error,
    error::{Error, Result},
    fetcher::{generic::ByteStream, Metadata},
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    operator: Operator,

    file_path: PathBuf,

    length: u64,
}

impl Fetcher {
    pub async fn new<S, T, U, V>(
        endpoint: S,
        user: T,
        identity_file: U,
        file_path: V,
    ) -> Result<Self>
    where
        S: fmt::Display + Send + Sync,
        T: fmt::Display + Send + Sync,
        U: fmt::Display + Send + Sync,
        V: fmt::Display + Send + Sync,
    {
        let mut builder = services::Sftp::default();
        let _ = builder
            .root("/")
            .endpoint(endpoint.to_string().as_str())
            .user(user.to_string().as_str())
            .key(dbg!(identity_file.to_string().as_str()))
            .known_hosts_strategy("Accept");
        let file_path = PathBuf::from(file_path.to_string());

        let operator =
            Operator::new(builder).with_context(|_| error::BuildOpenDALOperatorSnafu)?.finish();

        let metadata = operator
            .stat(&file_path.to_string_lossy())
            .await
            .with_context(|_| error::GetMetadataFromSftpSnafu)?;

        if metadata.is_dir() {
            return Err(Error::FetchingDirectory);
        }

        Ok(Self { operator, file_path, length: metadata.content_length() })
    }

    pub fn fetch_metadata(&self) -> Metadata {
        Metadata {
            length: self.length,
            filename: self
                .file_path
                .file_name()
                .map_or_else(|| PathBuf::from("index.html"), PathBuf::from),
        }
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        self.fetch_bytes(0, self.length - 1).await
    }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        let reader = self
            .operator
            .reader(&self.file_path.to_string_lossy())
            .await
            .context(error::CreateReaderSnafu)?;
        Ok(ByteStream::new(reader, start, end))
    }
}
