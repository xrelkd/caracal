use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::{Bytes, BytesMut};
use snafu::ResultExt;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt},
    sync::Mutex,
};

use crate::{error, error::Result, fetcher::Metadata};

const MAX_BUFFER_SIZE: usize = 1 << 10;

#[derive(Clone, Debug)]
pub struct Fetcher {
    file: Arc<Mutex<File>>,
    file_path: PathBuf,
    length: u64,
}

impl Fetcher {
    pub async fn new(url: reqwest::Url) -> Result<Self> {
        let file_path = PathBuf::from(url.path());
        let file = OpenOptions::new()
            .read(true)
            .open(&file_path)
            .await
            .with_context(|_| error::OpenFileSnafu { file_path: file_path.clone() })?;
        let metadata = file
            .metadata()
            .await
            .with_context(|_| error::GetFileLengthSnafu { file_path: file_path.clone() })?;

        Ok(Self { file: Arc::new(Mutex::new(file)), file_path, length: metadata.len() })
    }

    pub fn fetch_metadata(&self) -> Metadata {
        Metadata {
            length: self.length,
            filename: self
                .file_path
                .file_name()
                .map_or_else(|| PathBuf::from("index.html"), PathBuf::from),
        }
    }

    pub fn fetch_all(&mut self) -> ByteStream { self.fetch_bytes(0, self.length - 1) }

    pub fn fetch_bytes(&mut self, start: u64, end: u64) -> ByteStream {
        ByteStream::new(&self.file_path, self.file.clone(), start, end.min(self.length))
    }
}

pub struct ByteStream {
    file_path: PathBuf,
    file: Arc<Mutex<File>>,
    start: u64,
    end: u64,
}

impl ByteStream {
    pub fn new<P>(file_path: P, file: Arc<Mutex<File>>, start: u64, end: u64) -> Self
    where
        P: AsRef<Path>,
    {
        Self { file_path: file_path.as_ref().to_path_buf(), file, start, end }
    }

    pub async fn bytes(&mut self) -> Result<Option<Bytes>> {
        if self.start > self.end {
            return Ok(None);
        }

        let capacity = MAX_BUFFER_SIZE
            .min(usize::try_from(self.end - self.start + 1).unwrap_or(MAX_BUFFER_SIZE));
        let mut buf = BytesMut::zeroed(capacity);

        let mut file = self.file.lock().await;
        let _ = file
            .seek(SeekFrom::Start(self.start))
            .await
            .with_context(|_| error::SeekFileSnafu { file_path: self.file_path.clone() })?;
        let n = file
            .read_exact(buf.as_mut())
            .await
            .with_context(|_| error::ReadFileSnafu { file_path: self.file_path.clone() })?;
        drop(file);

        if n == 0 {
            Ok(None)
        } else {
            self.start += n as u64;
            Ok(Some(buf.freeze()))
        }
    }
}
