use std::{fmt, path::PathBuf};

use opendal::{services, Operator};
use snafu::ResultExt;

use crate::{
    error,
    error::Result,
    fetcher::{generic::ByteStream, Metadata},
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    operator: Operator,

    filename: String,
}

impl Fetcher {
    pub fn new<S, T, U, V, W>(
        endpoint_url: S,
        access_key: T,
        secret_key: U,
        bucket: V,
        filename: W,
    ) -> Result<Self>
    where
        S: fmt::Display,
        T: fmt::Display,
        U: fmt::Display,
        V: fmt::Display,
        W: fmt::Display,
    {
        let mut builder = services::S3::default();
        let _ = builder
            .region("auto")
            .endpoint(endpoint_url.to_string().as_str())
            .bucket(bucket.to_string().as_str())
            .access_key_id(access_key.to_string().as_str())
            .secret_access_key(secret_key.to_string().as_str());

        let operator =
            Operator::new(builder).with_context(|_| error::BuildOpenDALOperatorSnafu)?.finish();

        Ok(Self { operator, filename: filename.to_string() })
    }

    pub async fn fetch_metadata(&self) -> Result<Metadata> {
        let metadata = self
            .operator
            .stat(&self.filename)
            .await
            .with_context(|_| error::GetMetadataFromMinioSnafu)?;

        Ok(Metadata {
            length: metadata.content_length(),
            filename: metadata.content_disposition().map_or_else(
                || {
                    self.filename.split('/').collect::<Vec<_>>().last().map_or_else(
                        || PathBuf::from(caracal_base::FALLBACK_FILENAME),
                        PathBuf::from,
                    )
                },
                PathBuf::from,
            ),
        })
    }

    pub async fn fetch_bytes(&self, start: u64, end: u64) -> Result<ByteStream> {
        let reader = self.operator.reader(self.filename.as_str()).await.unwrap();
        Ok(ByteStream::new(reader, start, end))
    }

    pub async fn fetch_all(&self) -> Result<ByteStream> {
        let length = self.fetch_metadata().await?.length;
        self.fetch_bytes(0, length - 1).await
    }
}
