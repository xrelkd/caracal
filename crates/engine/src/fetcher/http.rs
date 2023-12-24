use bytes::Bytes;
use reqwest::{header, StatusCode};
use snafu::{OptionExt, ResultExt};

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
}

impl Fetcher {
    pub const fn new(client: reqwest::Client, url: reqwest::Url) -> Self { Self { client, url } }

    pub async fn fetch_metadata(&self) -> Result<Metadata> {
        let resp =
            self.client.head(self.url.clone()).send().await.context(error::FetchHttpHeaderSnafu)?;
        tracing::debug!("Response code: {}", resp.status());
        tracing::debug!("Received HEAD response: {:?}", resp.headers());

        if resp.status().is_success() {
            let len_str = resp
                .headers()
                .get(header::CONTENT_LENGTH)
                .context(error::NoLengthSnafu)?
                .to_str()
                .ok()
                .context(error::NoLengthSnafu)?;
            let length = len_str.parse::<u64>().with_context(|_| {
                error::ParseLengthFromHttpHeaderSnafu { value: len_str.to_string() }
            })?;
            if length == 0 {
                return Err(Error::NoLength);
            }
            let filename = resp.filename().unwrap_or_else(|| self.url.guess_filename());
            Ok(Metadata { length, filename })
        } else {
            let resp = self
                .client
                .get(self.url.clone())
                .header(header::RANGE, "0-0")
                .send()
                .await
                .context(error::FetchHttpHeaderSnafu)?;
            let resp_status = resp.status();
            tracing::debug!("Response code: {resp_status}");
            if resp_status.is_success() {
                tracing::debug!("Received GET 1B response: {:?}", resp.headers());

                let length = resp.content_length().context(error::NoLengthSnafu)?;
                if length == 0 {
                    return Err(Error::NoLength);
                }
                let file_name = resp.filename().unwrap_or_else(|| self.url.guess_filename());
                Ok(Metadata { length, filename: file_name })
            } else {
                match resp_status {
                    StatusCode::NOT_FOUND => Err(Error::NotFound { url: self.url.clone() }),
                    _ => Err(Error::UnknownHttpError { status_code: resp_status }),
                }
            }
        }
    }

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
