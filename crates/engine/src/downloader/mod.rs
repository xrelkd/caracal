mod chunk;
mod control_file;
mod factory;
mod progress_updater;
mod transfer_status;
mod worker;

use std::{collections::HashMap, io::SeekFrom, path::PathBuf, sync::Arc};

use futures::future;
use snafu::ResultExt;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{mpsc, oneshot, Mutex},
    task::{JoinHandle, JoinSet},
};

pub use self::{
    chunk::Chunk,
    factory::{Factory as DownloaderFactory, NewTask},
    transfer_status::TransferStatus,
};
use self::{
    control_file::ControlFile,
    progress_updater::ProgressUpdater,
    worker::{Worker, WorkerEvent},
};
use crate::{error, error::Error, fetcher::Fetcher, Progress};

type DownloaderHandle = Option<(mpsc::UnboundedSender<Event>, JoinHandle<Result<Summary, Error>>)>;

pub struct Downloader {
    use_simple: bool,
    worker_number: u64,
    transfer_status: TransferStatus,
    sink: File,
    source: Fetcher,
    uri: http::Uri,
    filename: PathBuf,
    handle: DownloaderHandle,
}

impl Downloader {
    /// # Errors
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.handle.is_none() {
            let sink_cloned = self.sink.try_clone().await.with_context(|_| {
                error::CloneFileInstanceSnafu { file_path: self.filename.clone() }
            })?;
            let (event_sender, event_receiver) = mpsc::unbounded_channel::<Event>();
            let join_handle = if self.use_simple {
                tokio::spawn(Self::serve_simple(
                    self.transfer_status.clone(),
                    sink_cloned,
                    self.source.clone(),
                    self.filename.clone(),
                    event_receiver,
                ))
            } else {
                let (control_file, transfer_status) =
                    ControlFile::new(&self.filename, self.uri.clone()).await?;
                if let Some(transfer_status) = transfer_status {
                    self.transfer_status = transfer_status;
                }
                tokio::spawn(Self::serve(
                    self.worker_number,
                    self.transfer_status.clone(),
                    Arc::new(Mutex::new(sink_cloned)),
                    self.source.clone(),
                    self.filename.clone(),
                    event_sender.clone(),
                    event_receiver,
                    control_file,
                ))
            };
            self.handle = Some((event_sender, join_handle));
        }
        Ok(())
    }

    /// # Errors
    pub async fn pause(&mut self) -> Result<(), Error> {
        if let Some((event_sender, join_handle)) = self.handle.take() {
            drop(event_sender.send(Event::Stop));
            match join_handle.await.context(error::JoinTaskSnafu)? {
                Ok(Summary::Completed { .. }) => {}
                Ok(Summary::Partial { transfer_status }) => {
                    self.transfer_status = transfer_status;
                }
                Err(err) => {
                    tracing::warn!("{err}");
                }
            }
        }
        Ok(())
    }

    /// # Errors
    pub async fn resume(&mut self) -> Result<(), Error> { self.start().await }

    /// # Errors
    pub async fn join(mut self) -> Result<Option<(TransferStatus, Progress)>, Error> {
        if let Some((_event_sender, join_handle)) = self.handle.take() {
            let transfer_status = match join_handle.await.context(error::JoinTaskSnafu)?? {
                Summary::Completed { transfer_status } | Summary::Partial { transfer_status } => {
                    transfer_status
                }
            };
            let mut progress = Progress::from(transfer_status.clone());
            progress.set_filename(self.filename.file_name().map_or_else(
                || caracal_base::FALLBACK_FILENAME,
                |s| s.to_str().unwrap_or(caracal_base::FALLBACK_FILENAME),
            ));
            Ok(Some((transfer_status, progress)))
        } else {
            Ok(None)
        }
    }

    pub async fn progress(&self) -> Option<Progress> {
        if let Some((event_sender, _join_handle)) = self.handle.as_ref() {
            let (send, recv) = oneshot::channel();
            drop(event_sender.send(Event::GetProgress(send)));
            recv.await
                .map(|status| {
                    let mut progress = Progress::from(status);
                    progress.set_filename(self.filename.file_name().map_or_else(
                        || caracal_base::FALLBACK_FILENAME,
                        |s| s.to_str().unwrap_or(caracal_base::FALLBACK_FILENAME),
                    ));
                    progress
                })
                .ok()
        } else {
            None
        }
    }

    pub fn add_worker(&self) {
        if let Some((event_sender, _join_handle)) = self.handle.as_ref() {
            drop(event_sender.send(Event::AddWorker));
        }
    }

    pub fn remove_worker(&self) {
        if let Some((event_sender, _join_handle)) = self.handle.as_ref() {
            drop(event_sender.send(Event::RemoveWorker));
        }
    }

    async fn serve_simple(
        mut transfer_status: TransferStatus,
        mut sink: File,
        mut source: Fetcher,
        file_path: PathBuf,
        mut event_receiver: mpsc::UnboundedReceiver<Event>,
    ) -> Result<Summary, Error> {
        let mut stream = source.fetch_all().await?;
        let _ = sink
            .seek(SeekFrom::Start(0))
            .await
            .with_context(|_| error::SeekFileSnafu { file_path: file_path.clone() })?;
        let mut received = 0;
        let mut summary = Summary::Partial { transfer_status: transfer_status.clone() };

        loop {
            let new_bytes = stream.bytes();
            let new_event = event_receiver.recv();
            futures::pin_mut!(new_bytes);
            futures::pin_mut!(new_event);

            match future::select(new_bytes, new_event).await {
                future::Either::Left((Ok(Some(bytes)), _)) => {
                    let _ = sink
                        .seek(SeekFrom::Start(received))
                        .await
                        .with_context(|_| error::SeekFileSnafu { file_path: file_path.clone() })?;
                    sink.write_all(bytes)
                        .await
                        .with_context(|_| error::WriteFileSnafu { file_path: file_path.clone() })?;
                    received += bytes.len() as u64;
                    transfer_status.update_progress(0, received);
                }
                future::Either::Left((Ok(None), _)) => {
                    if let Err(err) = sink.sync_all().await {
                        tracing::warn!("Error occurs while synchronizing file, error: {err}");
                    }
                    transfer_status.mark_chunk_completed(0);
                    summary = Summary::Completed { transfer_status };
                    break;
                }
                future::Either::Left((Err(_err), _)) => break,
                future::Either::Right((Some(Event::GetProgress(sender)), _)) => {
                    drop(sender.send(transfer_status.clone()));
                }
                future::Either::Right((..)) => break,
            }
        }

        Ok(summary)
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    async fn serve(
        worker_number: u64,
        mut transfer_status: TransferStatus,
        sink: Arc<Mutex<File>>,
        source: Fetcher,
        file_path: PathBuf,
        event_sender: mpsc::UnboundedSender<Event>,
        mut event_receiver: mpsc::UnboundedReceiver<Event>,
        mut control_file: ControlFile,
    ) -> Result<Summary, Error> {
        tracing::debug!("Start downloader with {worker_number} connection(s)");
        let (chunk_sender, chunk_receiver) = async_channel::unbounded::<Chunk>();

        let mut worker_event_senders = HashMap::new();
        let mut join_set = JoinSet::new();
        for id in 0..worker_number {
            let (worker_event_sender, worker_event_receiver) =
                mpsc::unbounded_channel::<WorkerEvent>();
            drop(worker_event_senders.insert(id, worker_event_sender));

            let worker = Worker {
                id,
                sink: sink.clone(),
                source: source.clone(),
                file_path: file_path.clone(),
                chunk_receiver: chunk_receiver.clone(),
                progress_updater: ProgressUpdater::from(event_sender.clone()),
                worker_event_receiver,
            };
            let _handle = join_set.spawn(worker.serve());
        }

        for chunk in transfer_status.chunks() {
            drop(chunk_sender.send(chunk).await);
        }

        let mut next_worker_id = worker_number;
        let mut chunk_to_worker = HashMap::new();

        let mut summary = Summary::Partial { transfer_status: transfer_status.clone() };
        while let Some(event) = event_receiver.recv().await {
            match event {
                Event::ChunkTransferStarted { worker_id, chunk_start } => {
                    let _unused = chunk_to_worker.insert(chunk_start, worker_id);
                }
                Event::ChunkTransferCompleted { chunk_start, worker_id: _worker_id } => {
                    let _unused = chunk_to_worker.remove(&chunk_start);
                    transfer_status.mark_chunk_completed(chunk_start);

                    if transfer_status.is_completed() {
                        summary = Summary::Completed { transfer_status };
                        control_file.remove().await;
                        break;
                    }
                }
                Event::UpdateChunkTranserProgress {
                    start,
                    received,
                    worker_id: _worker_id,
                    end: _end,
                } => {
                    transfer_status.update_progress(start, received);
                }
                Event::GetProgress(sender) => drop(sender.send(transfer_status.clone())),
                Event::Stop => {
                    control_file.update_progress(&transfer_status).await?;
                    control_file.flush().await?;
                    summary = Summary::Partial { transfer_status };
                    break;
                }
                Event::AddWorker => {
                    let (worker_event_sender, worker_event_receiver) =
                        mpsc::unbounded_channel::<WorkerEvent>();

                    drop(worker_event_senders.insert(next_worker_id, worker_event_sender));

                    let _handle = {
                        let worker = Worker {
                            id: next_worker_id,
                            chunk_receiver: chunk_receiver.clone(),
                            progress_updater: ProgressUpdater::from(event_sender.clone()),
                            sink: sink.clone(),
                            source: source.clone(),
                            file_path: file_path.clone(),
                            worker_event_receiver,
                        };

                        join_set.spawn(worker.serve())
                    };
                    next_worker_id += 1;

                    if let Some((origin_chunk, new_chunk)) = transfer_status.split() {
                        if let Some(worker_id) = chunk_to_worker.get(&origin_chunk.start) {
                            if let Some(worker) = worker_event_senders.get(worker_id) {
                                let (sender, wait) = oneshot::channel();
                                drop(worker.send(WorkerEvent::Stop(sender)));
                                let _ = wait.await;
                            }
                        }
                        let _ = chunk_sender.send(new_chunk).await;
                        let _ = chunk_sender.send(origin_chunk).await;
                    }
                }
                Event::RemoveWorker => {
                    if let Some((origin_chunk, new_chunk)) = transfer_status.freeze() {
                        if let Some(worker_id) = chunk_to_worker.get(&origin_chunk.start) {
                            if let Some(worker) = worker_event_senders.get(worker_id) {
                                let (sender, wait) = oneshot::channel();
                                drop(worker.send(WorkerEvent::Remove(sender)));
                                let _ = wait.await;
                            }
                        }
                        let _ = chunk_sender.send(new_chunk).await;
                    }
                }
            }
        }

        let _ = chunk_sender.close();

        let mut waiters = Vec::new();
        for (worker_id, sender) in worker_event_senders {
            let (send, wait) = oneshot::channel();
            waiters.push(wait);
            tracing::debug!("Shut down worker {worker_id}");
            drop(sender.send(WorkerEvent::Remove(send)));
        }

        drop(future::join_all(waiters).await);

        while join_set.join_next().await.is_some() {}

        {
            let mut sink = sink.lock().await;
            if let Err(err) = sink.flush().await {
                tracing::warn!("Error occurs while flushing file, error: {err}");
            }
            if let Err(err) = sink.sync_all().await {
                tracing::warn!("Error occurs while synchronizing file, error: {err}");
            }
            drop(sink);
        }

        Ok(summary)
    }
}

enum Event {
    Stop,
    GetProgress(oneshot::Sender<TransferStatus>),
    AddWorker,
    RemoveWorker,
    UpdateChunkTranserProgress { worker_id: u64, start: u64, end: u64, received: u64 },
    ChunkTransferStarted { worker_id: u64, chunk_start: u64 },
    ChunkTransferCompleted { worker_id: u64, chunk_start: u64 },
}

#[derive(Clone, Debug)]
enum Summary {
    Completed { transfer_status: TransferStatus },
    Partial { transfer_status: TransferStatus },
}
