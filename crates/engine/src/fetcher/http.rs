use std::path::PathBuf;

use opendal::{services, Operator};
use snafu::{OptionExt, ResultExt};

use crate::{
    error,
    error::Result,
    ext::UriExt,
    fetcher::{generic::ByteStream, Metadata},
};

#[derive(Clone, Debug)]
pub struct Fetcher {
    operator: Operator,

    file_path: String,

    metadata: Metadata,
}

impl Fetcher {
    pub async fn new(http_client: opendal::raw::HttpClient, uri: http::Uri) -> Result<Self> {
        let host = uri.host().context(error::HostnameNotProvidedSnafu)?;

        let mut builder = services::Http::default();
        let _ = builder.http_client(http_client);
        let scheme = uri.scheme_str().unwrap_or("https");
        if let Some(port) = uri.port() {
            let _ = builder.endpoint(&format!("{scheme}://{host}:{port}"));
        } else {
            let _ = builder.endpoint(&format!("{scheme}://{host}"));
        }
        let _ = builder.root("/");
        let file_path = {
            let path = uri.path_and_query().map_or("/", http::uri::PathAndQuery::as_str);
            if path == "/" {
                String::from("index.html")
            } else {
                path.to_string()
            }
        };

        let operator =
            Operator::new(builder).with_context(|_| error::BuildOpenDALOperatorSnafu)?.finish();

        let metadata = {
            let metadata = operator
                .stat(file_path.as_ref())
                .await
                .with_context(|_| error::GetMetadataFromHttpSnafu)?;
            let filename = metadata
                .content_disposition()
                .map(|content_disposition| {
                    extract_filename_from_content_disposition(content_disposition)
                        .unwrap_or_else(|| uri.guess_filename())
                })
                .map_or_else(|| uri.guess_filename(), PathBuf::from);
            let length = metadata.content_length();
            Metadata { length, filename }
        };

        Ok(Self { operator, file_path, metadata })
    }

    #[inline]
    pub const fn supports_range_request(&self) -> bool { self.metadata.length != 0 }

    pub fn fetch_metadata(&self) -> Metadata { self.metadata.clone() }

    pub async fn fetch_all(&self) -> Result<ByteStream> {
        self.operator
            .reader_with(&self.file_path)
            .await
            .map(ByteStream::from)
            .context(error::CreateReaderSnafu)
    }

    pub async fn fetch_bytes(&self, start: u64, end: u64) -> Result<ByteStream> {
        self.operator
            .reader_with(&self.file_path)
            .range(start..=end)
            .await
            .map(ByteStream::from)
            .context(error::CreateReaderSnafu)
    }
}

fn extract_filename_from_content_disposition(content_disposition: &str) -> Option<PathBuf> {
    let content_disposition = mailparse::parse_content_disposition(content_disposition);
    if let Some(value) = content_disposition.params.get("filename*") {
        let mut parts = value.split("UTF-8''");
        let _ = parts.next();
        if let Some(part) = parts.next() {
            if let Ok(s) = urlencoding::decode(part) {
                if !s.is_empty() {
                    return Some(PathBuf::from(s.to_string()));
                }
            }
        }
    }
    content_disposition.params.get("filename").map(PathBuf::from)
}
