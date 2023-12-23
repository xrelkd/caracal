use std::fmt;

use crate::downloader::{Chunk, TransferStatus};

#[derive(Clone, Debug)]
pub struct Progress {
    filename: String,
    content_length: u64,
    chunks: Vec<ProgressChunk>,
}

impl Progress {
    #[must_use]
    pub fn new() -> Self { Self { filename: String::new(), content_length: 0, chunks: Vec::new() } }

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

    #[must_use]
    pub fn is_completed(&self) -> bool { self.remaining() == 0 }

    #[must_use]
    pub fn total_received(&self) -> u64 { self.chunks.iter().map(|chunk| chunk.received).sum() }

    #[must_use]
    pub const fn content_length(&self) -> u64 { self.content_length }

    #[must_use]
    pub fn filename(&self) -> &str { self.filename.as_str() }

    pub fn set_filename<S>(&mut self, filename: S)
    where
        S: fmt::Display,
    {
        self.filename = filename.to_string();
    }
}

impl Default for Progress {
    fn default() -> Self { Self::new() }
}

impl From<TransferStatus> for Progress {
    fn from(status: TransferStatus) -> Self {
        let chunks = status.chunks().into_iter().map(ProgressChunk::from).collect();
        Self { filename: String::new(), content_length: status.content_length(), chunks }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgressChunk {
    pub start: u64,
    pub end: u64,
    pub received: u64,
}

impl ProgressChunk {
    pub const fn len(&self) -> u64 { self.end - self.start + 1 }

    pub const fn remaining(&self) -> u64 {
        let len = self.len();
        debug_assert!(len >= self.received);
        len - self.received
    }

    pub const fn is_completed(&self) -> bool { self.remaining() == 0 }
}

impl From<Chunk> for ProgressChunk {
    fn from(Chunk { start, end, received }: Chunk) -> Self { Self { start, end, received } }
}
