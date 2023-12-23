use std::path::PathBuf;

use reqwest::header;

pub trait HttpResponseExt {
    fn filename(&self) -> Option<PathBuf>;
}

impl HttpResponseExt for reqwest::Response {
    fn filename(&self) -> Option<PathBuf> {
        for token in self.headers().get_all(header::CONTENT_DISPOSITION) {
            if let Ok(kv) = token.to_str().map(str::to_ascii_uppercase) {
                if let Some((key, value)) = kv.find('=').map(|idx| {
                    let key = kv[0..idx].trim().to_lowercase();
                    let mut value = kv[idx + 1..].trim();
                    if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
                        value = &value[1..value.len() - 1];
                    }
                    (key, value.to_string())
                }) {
                    if key == "filename*" {
                        let mut parts = value.split("utf-8''");
                        if let Some(part) = parts.next() {
                            return urlencoding::decode(part)
                                .ok()
                                .map(|s| PathBuf::from(s.to_string()));
                        }
                    } else if key == "filename" {
                        return Some(PathBuf::from(value));
                    }
                }
            }
        }
        None
    }
}
