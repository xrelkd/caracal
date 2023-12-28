use std::path::{Path, PathBuf};

pub trait PathExt {
    fn file_name_or_fallback(&self) -> PathBuf;
}

impl PathExt for Path {
    fn file_name_or_fallback(&self) -> PathBuf {
        self.file_name().map_or_else(
            || PathBuf::from(caracal_base::FALLBACK_FILENAME),
            |s| PathBuf::from(s.to_str().unwrap_or(caracal_base::FALLBACK_FILENAME)),
        )
    }
}

impl PathExt for PathBuf {
    fn file_name_or_fallback(&self) -> PathBuf {
        let r: &Path = self.as_ref();
        r.file_name_or_fallback()
    }
}
