mod fs;
mod generic;
mod http;
mod minio;

use std::{fmt, path::PathBuf};

use bytes::Bytes;

use crate::error::Result;

#[derive(Clone, Debug)]
pub struct Metadata {
    pub length: u64,

    pub filename: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Fetcher {
    FileSystem(fs::Fetcher),
    Http(http::Fetcher),
    Minio(minio::Fetcher),
}

impl Fetcher {
    pub async fn new_file(url: reqwest::Url) -> Result<Self> {
        Ok(Self::FileSystem(fs::Fetcher::new(url).await?))
    }

    pub const fn new_http(client: reqwest::Client, url: reqwest::Url) -> Self {
        Self::Http(http::Fetcher::new(client, url))
    }

    pub fn new_minio<S, T, U, V, W>(
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
        Ok(Self::Minio(minio::Fetcher::new(
            endpoint_url,
            access_key,
            secret_key,
            bucket,
            filename,
        )?))
    }

    pub async fn fetch_metadata(&self) -> Result<Metadata> {
        match self {
            Self::FileSystem(fetcher) => Ok(fetcher.fetch_metadata()),
            Self::Http(fetcher) => fetcher.fetch_metadata().await,
            Self::Minio(fetcher) => fetcher.fetch_metadata().await,
        }
    }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        debug_assert!(start <= end);
        match self {
            Self::Http(client) => client.fetch_bytes(start, end).await.map(ByteStream::Http),
            Self::Minio(client) => client.fetch_bytes(start, end).await.map(ByteStream::Generic),
            Self::FileSystem(client) => {
                client.fetch_bytes(start, end).await.map(ByteStream::Generic)
            }
        }
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        match self {
            Self::Http(client) => client.fetch_all().await.map(ByteStream::Http),
            Self::Minio(client) => client.fetch_all().await.map(ByteStream::Generic),
            Self::FileSystem(client) => client.fetch_all().await.map(ByteStream::Generic),
        }
    }
}

pub enum ByteStream {
    Http(http::ByteStream),
    Generic(generic::ByteStream),
}

impl ByteStream {
    pub async fn bytes(&mut self) -> Result<Option<Bytes>> {
        match self {
            Self::Http(stream) => stream.bytes().await,
            Self::Generic(stream) => stream.bytes().await,
        }
    }
}
