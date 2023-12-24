use std::path::PathBuf;

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
    pub async fn new(url: reqwest::Url) -> Result<Self> {
        let mut builder = services::Fs::default();
        let _ = builder.root("/");
        let file_path = PathBuf::from(url.path());

        let operator =
            Operator::new(builder).with_context(|_| error::BuildOpenDALOperatorSnafu)?.finish();

        let metadata = operator
            .stat(&file_path.to_string_lossy())
            .await
            .with_context(|_| error::GetMetadataFromMinioSnafu)?;

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
