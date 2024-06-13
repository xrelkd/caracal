use std::{io::SeekFrom, path::PathBuf, sync::Arc};

use futures::future;
use snafu::ResultExt;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{mpsc, oneshot, Mutex},
};

use crate::{
    downloader::{Chunk, ProgressUpdater},
    error,
    error::Error,
    fetcher::Fetcher,
};

pub struct Worker {
    pub id: u64,
    pub sink: Arc<Mutex<File>>,
    pub source: Fetcher,
    pub file_path: PathBuf,
    pub chunk_receiver: async_channel::Receiver<Chunk>,
    pub event_receiver: mpsc::UnboundedReceiver<WorkerEvent>,
    pub progress_updater: ProgressUpdater,
}

impl Worker {
    pub async fn serve(self) -> Result<(), Error> {
        let Self {
            id,
            mut source,
            sink,
            file_path,
            chunk_receiver,
            mut event_receiver,
            progress_updater,
        } = self;

        while let Ok(chunk) = chunk_receiver.recv().await {
            tracing::debug!(
                "Transfer chunk in range {}-{}, received: {}, length: {}, worker: {id}",
                chunk.start,
                chunk.end,
                chunk.received,
                chunk.len()
            );
            progress_updater.signal_started(id, chunk.start);
            if chunk.received >= chunk.len() {
                progress_updater.signal_completed(id, chunk.start);
                continue;
            }
            let mut received = chunk.received;
            let mut stream = source.fetch_bytes(chunk.start + chunk.received, chunk.end).await?;

            loop {
                let new_bytes = stream.bytes();
                let new_event = event_receiver.recv();
                futures::pin_mut!(new_bytes);
                futures::pin_mut!(new_event);

                match future::select(new_bytes, new_event).await {
                    future::Either::Left((Ok(Some(bytes)), _)) => {
                        {
                            let mut sink = sink.lock().await;
                            let _ = sink
                                .seek(SeekFrom::Start(chunk.start + received))
                                .await
                                .with_context(|_| error::SeekFileSnafu {
                                    file_path: file_path.clone(),
                                })?;
                            received += if bytes.len() as u64 + chunk.start <= chunk.end {
                                sink.write_all(bytes).await.with_context(|_| {
                                    error::WriteFileSnafu { file_path: file_path.clone() }
                                })?;
                                bytes.len() as u64
                            } else {
                                let n = usize::try_from(chunk.remaining()).unwrap_or_default();
                                if n > 0 {
                                    sink.write_all(&bytes[..n]).await.with_context(|_| {
                                        error::WriteFileSnafu { file_path: file_path.clone() }
                                    })?;
                                }
                                n as u64
                            };
                            drop(sink);
                        }
                        progress_updater.update(id, chunk.start, chunk.end, received);
                    }
                    future::Either::Left((Ok(None), _)) => {
                        progress_updater.signal_completed(id, chunk.start);
                        break;
                    }
                    future::Either::Left((Err(err), _)) => {
                        tracing::warn!("{err}");
                        break;
                    }
                    future::Either::Right((Some(WorkerEvent::Remove(sender)), _)) => {
                        let _ = sender.send(());
                        return Ok(());
                    }
                    future::Either::Right((Some(WorkerEvent::Stop(sender)), _)) => {
                        let _ = sender.send(());
                        break;
                    }
                    future::Either::Right((None, _)) => break,
                }
            }
        }

        Ok(())
    }
}

pub enum WorkerEvent {
    Remove(oneshot::Sender<()>),
    Stop(oneshot::Sender<()>),
}
