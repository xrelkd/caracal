use std::collections::HashMap;

use crate::{downloader::Chunk, error::Error};

#[derive(Clone, Debug)]
pub struct TransferStatus {
    pub content_length: u64,

    pub chunks: HashMap<u64, Chunk>,
}

impl TransferStatus {
    pub fn new(content_length: u64, chunk_size: u64) -> Result<Self, Error> {
        let chunks = InitialChunks::new(0, content_length - 1, chunk_size)?
            .map(|chunk| (chunk.start, chunk))
            .collect();
        Ok(Self { content_length, chunks })
    }

    pub fn unknown_length() -> Self {
        let chunks = HashMap::from([(0, Chunk { start: 0, end: 0, received: 0 })]);
        Self { content_length: 0, chunks }
    }

    pub fn chunks(&self) -> Vec<Chunk> {
        let mut chunks = self.chunks.values().cloned().collect::<Vec<_>>();
        chunks.sort_unstable();
        chunks
    }

    pub fn update_progress(&mut self, id: u64, received: u64) {
        let _unused = self.chunks.get_mut(&id).map(|chunk| chunk.received = received);
    }

    pub fn is_completed(&self) -> bool { self.remaining() == 0 }

    pub fn total_received(&self) -> u64 { self.chunks.values().map(|chunk| chunk.received).sum() }

    pub const fn content_length(&self) -> u64 { self.content_length }

    pub fn remaining(&self) -> u64 { self.content_length - self.total_received() }

    pub fn split(&mut self) -> Option<(Chunk, Chunk)> {
        if self.is_completed() {
            None
        } else {
            let mut chunks: Vec<_> = self.chunks.values_mut().collect();
            chunks.sort_unstable_by_key(|c| c.remaining());
            let (origin_chunk, new_chunk) = if let Some(origin_chunk) = chunks.pop() {
                let new_chunk = origin_chunk.split();
                (origin_chunk.clone(), new_chunk)
            } else {
                return None;
            };
            if let Some(new_chunk) = new_chunk {
                let _ = self.chunks.insert(new_chunk.start, new_chunk.clone());
                Some((origin_chunk, new_chunk))
            } else {
                None
            }
        }
    }

    pub fn freeze(&mut self) -> Option<(Chunk, Chunk)> {
        if self.is_completed() {
            None
        } else {
            let mut chunks: Vec<_> = self.chunks.values_mut().collect();
            chunks.sort_unstable_by_key(|c| c.remaining());
            let (origin_chunk, new_chunk) = if let Some(origin_chunk) = chunks.pop() {
                let new_chunk = origin_chunk.freeze();
                (origin_chunk.clone(), new_chunk)
            } else {
                return None;
            };
            if let Some(new_chunk) = new_chunk {
                let _ = self.chunks.insert(new_chunk.start, new_chunk.clone());
                Some((origin_chunk, new_chunk))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
struct InitialChunks {
    start: u64,
    end: u64,
    chunk_size: u64,
}

impl InitialChunks {
    /// Create the iterator
    /// # Arguments
    /// * `start` - the first byte of the file, typically 0
    /// * `end` - the highest value in bytes, typically content-length - 1
    /// * `chunk_size` - the desired size of the chunks
    pub const fn new(start: u64, end: u64, chunk_size: u64) -> Result<Self, Error> {
        if chunk_size == 0 {
            return Err(Error::BadChunkSize { value: chunk_size });
        }

        Ok(Self { start, end, chunk_size })
    }
}

impl Iterator for InitialChunks {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += self.chunk_size.min(self.end - self.start + 1);
            Some(Chunk { start: prev_start, end: self.start - 1, received: 0 })
        }
    }
}
