use std::path::PathBuf;

use reqwest::header;

pub trait HttpResponseExt {
    fn filename(&self) -> Option<PathBuf>;
}

impl HttpResponseExt for reqwest::Response {
    fn filename(&self) -> Option<PathBuf> {
        for token in self.headers().get_all(header::CONTENT_DISPOSITION) {
            if let Ok(kv) = token.to_str() {
                let content_disposition = mailparse::parse_content_disposition(kv);
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
                if let Some(value) = content_disposition.params.get("filename") {
                    return Some(PathBuf::from(value));
                }
            }
        }
        None
    }
}
