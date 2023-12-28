use std::path::{Path, PathBuf};

use crate::{
    downloader::{Chunk, TransferStatus},
    ext::PathExt,
};

#[derive(Clone, Debug)]
pub struct DownloaderStatus {
    file_path: PathBuf,

    content_length: u64,

    chunks: Vec<ProgressChunk>,

    concurrent_number: usize,
}

impl DownloaderStatus {
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
            content_length: 0,
            chunks: Vec::new(),
            concurrent_number: 0,
        }
    }

    #[must_use]
    pub fn chunks(&self) -> Vec<ProgressChunk> { self.chunks.clone() }

    #[must_use]
    pub fn total_chunk_count(&self) -> usize { self.chunks.len() }

    #[must_use]
    pub fn completed_chunk_count(&self) -> usize {
        self.chunks.iter().filter(|c| c.is_completed()).count()
    }

    #[must_use]
    pub fn remaining(&self) -> u64 {
        let total_received = self.total_received();
        if self.content_length < total_received {
            0
        } else {
            self.content_length - total_received
        }
    }

    #[inline]
    #[must_use]
    pub fn is_completed(&self) -> bool { self.chunks.iter().all(ProgressChunk::is_completed) }

    #[inline]
    #[must_use]
    pub fn total_received(&self) -> u64 { self.chunks.iter().map(|chunk| chunk.received).sum() }

    #[inline]
    #[must_use]
    pub const fn content_length(&self) -> u64 { self.content_length }

    #[inline]
    #[must_use]
    pub fn filename(&self) -> PathBuf { self.file_path.file_name_or_fallback() }

    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &Path { &self.file_path }

    pub fn set_file_path<P>(&mut self, file_path: P)
    where
        P: AsRef<Path>,
    {
        self.file_path = file_path.as_ref().to_path_buf();
    }

    #[must_use]
    pub const fn concurrent_number(&self) -> usize { self.concurrent_number }
}

impl Default for DownloaderStatus {
    fn default() -> Self { Self::new() }
}

impl From<TransferStatus> for DownloaderStatus {
    fn from(status: TransferStatus) -> Self {
        let chunks = status.chunks().into_iter().map(ProgressChunk::from).collect();
        Self {
            file_path: PathBuf::new(),
            content_length: status.content_length(),
            chunks,
            concurrent_number: status.concurrent_number(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgressChunk {
    pub start: u64,

    pub end: u64,

    pub received: u64,

    pub is_completed: bool,
}

impl ProgressChunk {
    pub const fn len(&self) -> u64 { self.end - self.start + 1 }

    pub const fn remaining(&self) -> u64 {
        let len = self.len();
        if len >= self.received {
            len - self.received
        } else {
            0
        }
    }

    pub const fn is_completed(&self) -> bool { self.is_completed }
}

impl From<Chunk> for ProgressChunk {
    fn from(Chunk { start, end, received, is_completed }: Chunk) -> Self {
        Self { start, end, received, is_completed }
    }
}
