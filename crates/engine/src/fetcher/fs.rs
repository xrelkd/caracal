use std::path::{Path, PathBuf};

use opendal::{services, Operator};
use snafu::ResultExt;

use crate::{
    error,
    error::{Error, Result},
    ext::PathExt,
    fetcher::{generic::ByteStream, Metadata},
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    operator: Operator,

    file_path: PathBuf,

    length: u64,
}

impl Fetcher {
    pub async fn new<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path> + Send + Sync,
    {
        let mut builder = services::Fs::default();
        let _ = builder.root("/");
        let file_path = file_path.as_ref().to_path_buf();

        let operator =
            Operator::new(builder).with_context(|_| error::BuildOpenDALOperatorSnafu)?.finish();

        let metadata = operator
            .stat(&file_path.to_string_lossy())
            .await
            .with_context(|_| error::GetMetadataFromFileSystemSnafu)?;

        if metadata.is_dir() {
            return Err(Error::FetchingDirectory);
        }

        Ok(Self { operator, file_path, length: metadata.content_length() })
    }

    pub fn fetch_metadata(&self) -> Metadata {
        Metadata { length: self.length, filename: self.file_path.file_name_or_fallback() }
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        self.fetch_bytes(0, self.length - 1).await
    }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        self.operator
            .reader_with(&self.file_path.to_string_lossy())
            .range(start..=end)
            .await
            .map(ByteStream::from)
            .context(error::CreateReaderSnafu)
    }
}
