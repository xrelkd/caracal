use std::path::PathBuf;

use crate::minio::MinioPath;

pub trait UrlExt {
    const FALLBACK_FILENAME: &'static str = caracal_base::FALLBACK_FILENAME;

    fn guess_filename(&self) -> PathBuf;

    fn minio_path(&self) -> Option<MinioPath>;
}

impl UrlExt for reqwest::Url {
    fn guess_filename(&self) -> PathBuf {
        PathBuf::from(self.path())
            .file_name()
            .map_or_else(|| PathBuf::from(Self::FALLBACK_FILENAME), PathBuf::from)
    }

    fn minio_path(&self) -> Option<MinioPath> {
        if self.scheme() != "minio" {
            return None;
        }

        let alias = self.host_str()?.to_string();
        let path_parts = self.path().split('/').collect::<Vec<_>>();
        if path_parts.len() < 3 {
            return None;
        }
        let bucket = path_parts[1].to_string();
        let object = path_parts[2..].join("/");

        Some(MinioPath { alias, bucket, object })
    }
}
