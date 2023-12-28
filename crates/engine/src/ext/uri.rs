use std::path::PathBuf;

use caracal_base::profile::minio::MinioPath;

use crate::ext::PathExt;

pub trait UriExt {
    fn guess_filename(&self) -> PathBuf;

    fn minio_path(&self) -> Option<MinioPath>;
}

impl UriExt for http::Uri {
    fn guess_filename(&self) -> PathBuf { PathBuf::from(self.path()).file_name_or_fallback() }

    fn minio_path(&self) -> Option<MinioPath> {
        if self.scheme_str() != Some("minio") {
            return None;
        }

        let alias = self.host()?.to_string();
        let path_parts = self.path().split('/').collect::<Vec<_>>();
        if path_parts.len() < 3 {
            return None;
        }
        let bucket = path_parts[1].to_string();
        let object = path_parts[2..].join("/");

        Some(MinioPath { alias, bucket, object })
    }
}
