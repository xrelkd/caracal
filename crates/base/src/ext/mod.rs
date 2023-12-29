use crate::model;

pub trait ProgressChunks {
    fn received_bytes(&self) -> u64;

    fn total_bytes(&self) -> u64;

    fn is_completed(&self) -> bool;
}

impl ProgressChunks for Vec<model::ProgressChunk> {
    #[inline]
    fn received_bytes(&self) -> u64 { self.iter().map(|chunk| chunk.received).sum() }

    #[inline]
    fn total_bytes(&self) -> u64 { self.iter().map(model::ProgressChunk::len).sum() }

    #[inline]
    fn is_completed(&self) -> bool { self.iter().all(model::ProgressChunk::is_completed) }
}
