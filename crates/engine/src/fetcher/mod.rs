mod fs;
mod http;

use std::path::PathBuf;

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
}

impl Fetcher {
    pub async fn new_file(url: reqwest::Url) -> Result<Self> {
        Ok(Self::FileSystem(fs::Fetcher::new(url).await?))
    }

    pub const fn new_http(client: reqwest::Client, url: reqwest::Url) -> Self {
        Self::Http(http::Fetcher::new(client, url))
    }

    pub async fn fetch_metadata(&self) -> Result<Metadata> {
        match self {
            Self::FileSystem(fetcher) => Ok(fetcher.fetch_metadata()),
            Self::Http(fetcher) => fetcher.fetch_metadata().await,
        }
    }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        debug_assert!(start <= end);
        match self {
            Self::FileSystem(client) => Ok(ByteStream::FileSystem(client.fetch_bytes(start, end))),
            Self::Http(client) => client.fetch_bytes(start, end).await.map(ByteStream::Http),
        }
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        match self {
            Self::FileSystem(client) => Ok(ByteStream::FileSystem(client.fetch_all())),
            Self::Http(client) => client.fetch_all().await.map(ByteStream::Http),
        }
    }
}

pub enum ByteStream {
    FileSystem(fs::ByteStream),
    Http(http::ByteStream),
}

impl ByteStream {
    pub async fn bytes(&mut self) -> Result<Option<Bytes>> {
        match self {
            Self::FileSystem(stream) => stream.bytes().await,
            Self::Http(stream) => stream.bytes().await,
        }
    }
}
