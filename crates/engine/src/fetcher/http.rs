use bytes::Bytes;
use reqwest::{header, StatusCode};
use snafu::ResultExt;

use crate::{
    error,
    error::{Error, Result},
    ext::{HttpResponseExt, UriExt},
    fetcher::Metadata,
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    client: reqwest::Client,
    uri: http::Uri,
    metadata: Metadata,
}

impl Fetcher {
    pub async fn new(client: reqwest::Client, uri: http::Uri) -> Result<Self> {
        let resp =
            client.head(uri.to_string()).send().await.context(error::FetchHttpHeaderSnafu)?;
        tracing::debug!("Response code: {}", resp.status());
        tracing::debug!("Received HEAD response: {:?}", resp.headers());

        let metadata = if resp.status().is_success() {
            let length = resp.headers().get(header::CONTENT_LENGTH).map_or(0, |len_str| {
                len_str.to_str().map_or(0, |len_str| len_str.parse::<u64>().unwrap_or_default())
            });

            let filename = resp.filename().unwrap_or_else(|| uri.guess_filename());
            Metadata { length, filename }
        } else {
            let resp = client
                .get(uri.to_string())
                .header(header::RANGE, "0-0")
                .send()
                .await
                .context(error::FetchHttpHeaderSnafu)?;
            let resp_status = resp.status();
            tracing::debug!("Response code: {resp_status}");
            if resp_status.is_success() {
                tracing::debug!("Received GET 1B response: {:?}", resp.headers());

                let length = resp.content_length().unwrap_or(0);
                let filename = resp.filename().unwrap_or_else(|| uri.guess_filename());
                Metadata { length, filename }
            } else {
                return match resp_status {
                    StatusCode::NOT_FOUND => Err(Error::NotFound { uri: uri.clone() }),
                    _ => Err(Error::UnknownHttpError { status_code: resp_status }),
                };
            }
        };

        Ok(Self { client, uri, metadata })
    }

    #[inline]
    pub const fn supports_range_request(&self) -> bool { self.metadata.length != 0 }

    pub fn fetch_metadata(&self) -> Metadata { self.metadata.clone() }

    pub async fn fetch_bytes(&mut self, start: u64, end: u64) -> Result<ByteStream> {
        let resp = self
            .client
            .get(self.uri.to_string())
            .header(header::RANGE, format!("bytes={start}-{end}"))
            .send()
            .await
            .context(error::FetchRangeFromHttpSnafu)?;
        let content_length = end - start + 1;
        dbg!((start, end, content_length, resp.status(), resp.headers()));
        assert_eq!(resp.status(), reqwest::StatusCode::PARTIAL_CONTENT);
        Ok(ByteStream::new(resp, content_length))
    }

    pub async fn fetch_all(&mut self) -> Result<ByteStream> {
        self.client
            .get(self.uri.to_string())
            .send()
            .await
            .context(error::FetchRangeFromHttpSnafu)
            .map(ByteStream::from)
    }
}

#[derive(Debug)]
pub struct ByteStream {
    response: reqwest::Response,
    received: u64,
    content_length: Option<u64>,
    buffer: Bytes,
}

impl ByteStream {
    pub fn new(response: reqwest::Response, content_length: u64) -> Self {
        Self { response, received: 0, content_length: Some(content_length), buffer: Bytes::new() }
    }

    pub async fn bytes(&mut self) -> Result<Option<&[u8]>> {
        match self.response.chunk().await.context(error::FetchBytesFromHttpSnafu) {
            Ok(Some(bytes)) => {
                if let Some(content_length) = self.content_length {
                    if self.received + bytes.len() as u64 > content_length {
                        let remaining =
                            usize::try_from(content_length - self.received).unwrap_or_default();
                        dbg!((
                            content_length,
                            self.received,
                            remaining,
                            bytes.len(),
                            self.response.status(),
                            self.response.headers()
                        ));
                        assert_eq!(self.response.status(), reqwest::StatusCode::PARTIAL_CONTENT);
                        if remaining == 0 {
                            return Ok(None);
                        }
                        self.received += remaining as u64;
                        self.buffer = bytes.slice(0..remaining);
                        assert_eq!(self.buffer.len(), remaining);
                    } else {
                        self.received += bytes.len() as u64;
                        self.buffer = bytes;
                    }
                    Ok(Some(self.buffer.as_ref()))
                } else {
                    self.received += bytes.len() as u64;
                    self.buffer = bytes;
                    Ok(Some(self.buffer.as_ref()))
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl From<reqwest::Response> for ByteStream {
    fn from(response: reqwest::Response) -> Self {
        let content_length = response.content_length();
        Self { response, received: 0, content_length, buffer: Bytes::new() }
    }
}
