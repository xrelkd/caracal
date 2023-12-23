use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{downloader, downloader::TransferStatus};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Control {
    pub schema: u32,

    pub uris: Vec<String>,

    pub content_length: Option<u64>,

    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chunk {
    pub start: u64,

    pub end: u64,

    pub received: u64,
}

impl From<Chunk> for downloader::Chunk {
    fn from(Chunk { start, end, received }: Chunk) -> Self { Self { start, end, received } }
}

impl From<downloader::Chunk> for Chunk {
    fn from(downloader::Chunk { start, end, received }: downloader::Chunk) -> Self {
        Self { start, end, received }
    }
}

impl From<Control> for TransferStatus {
    fn from(Control { content_length, chunks, .. }: Control) -> Self {
        let content_length = content_length.unwrap_or(0);
        let chunks = chunks
            .into_iter()
            .map(|chunk| (chunk.start, downloader::Chunk::from(chunk)))
            .collect::<HashMap<_, _>>();
        Self { content_length, chunks }
    }
}
