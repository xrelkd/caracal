use std::path::PathBuf;

pub trait UrlExt {
    const FALLBACK_FILENAME: &'static str = "index.html";

    fn guess_filename(&self) -> PathBuf;
}

impl UrlExt for reqwest::Url {
    fn guess_filename(&self) -> PathBuf {
        PathBuf::from(self.path())
            .file_name()
            .map_or_else(|| PathBuf::from(Self::FALLBACK_FILENAME), PathBuf::from)
    }
}
