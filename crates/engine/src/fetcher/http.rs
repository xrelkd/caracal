use bytes::Bytes;
use reqwest::{header, StatusCode};
use snafu::ResultExt;

use crate::{
    error,
    error::{Error, Result},
    ext::{HttpResponseExt, UrlExt},
    fetcher::Metadata,
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    client: reqwest::Client,
    url: reqwest::Url,
    metadata: Metadata,
}

impl Fetcher {
    pub async fn new(client: reqwest::Client, url: reqwest::Url) -> Result<Self> {
        let resp = client.head(url.clone()).send().await.context(error::FetchHttpHeaderSnafu)?;
        tracing::debug!("Response code: {}", resp.status());
        tracing::debug!("Received HEAD response: {:?}", resp.headers());

        let metadata = if resp.status().is_success() {
            let length = resp.headers().get(header::CONTENT_LENGTH).map_or(0, |len_str| {
                len_str.to_str().map_or(0, |len_str| len_str.parse::<u64>().unwrap_or_default())
            });

            let filename = resp.filename().unwrap_or_else(|| url.guess_filename());
            Metadata { length, filename }
        } else {
            let resp = client
                .get(url.clone())
                .header(header::RANGE, "0-0")
                .send()
                .await
                .context(error::FetchHttpHeaderSnafu)?;
            let resp_status = resp.status();
            tracing::debug!("Response code: {resp_status}");
            if resp_status.is_success() {
                tracing::debug!("Received GET 1B response: {:?}", resp.headers());

                let length = resp.content_length().unwrap_or(0);
                let filename = resp.filename().unwrap_or_else(|| url.guess_filename());
                Metadata { length, filename }
            } else {
                return match resp_status {
                    StatusCode::NOT_FOUND => Err(Error::NotFound { url: url.clone() }),
                    _ => Err(Error::UnknownHttpError { status_code: resp_status }),
                };
            }
        };

        Ok(Self { client, url, metadata })
    }

    pub fn fetch_metadata(&self) -> Metadata { self.metadata.clone() }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        let resp = self
            .client
            .get(self.url.clone())
            .header(header::RANGE, format!("bytes={start}-{end}"))
            .send()
            .await
            .context(error::FetchRangeFromHttpSnafu)?;
        Ok(ByteStream::from(resp))
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        self.client
            .get(self.url.clone())
            .send()
            .await
            .context(error::FetchRangeFromHttpSnafu)
            .map(ByteStream::from)
    }
}

#[derive(Debug)]
pub struct ByteStream(reqwest::Response);

impl ByteStream {
    pub async fn bytes(&mut self) -> Result<Option<Bytes>> {
        self.0.chunk().await.context(error::FetchBytesFromHttpSnafu)
    }
}

impl From<reqwest::Response> for ByteStream {
    fn from(response: reqwest::Response) -> Self { Self(response) }
}
