use tokio::sync::mpsc;

use crate::downloader::Event;

#[derive(Clone)]
pub struct ProgressUpdater(mpsc::UnboundedSender<Event>);

impl From<mpsc::UnboundedSender<Event>> for ProgressUpdater {
    fn from(sender: mpsc::UnboundedSender<Event>) -> Self { Self(sender) }
}

impl ProgressUpdater {
    pub fn signal_started(&self, worker_id: u64, chunk_start: u64) {
        drop(self.0.send(Event::ChunkTransferStarted { worker_id, chunk_start }));
    }

    pub fn signal_completed(&self, worker_id: u64, chunk_start: u64) {
        drop(self.0.send(Event::ChunkTransferCompleted { worker_id, chunk_start }));
    }

    pub fn update(&self, worker_id: u64, start: u64, end: u64, received: u64) {
        drop(self.0.send(Event::UpdateChunkTranserProgress { worker_id, start, end, received }));
    }
}
