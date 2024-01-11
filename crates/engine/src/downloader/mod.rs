mod chunk;
mod control_file;
mod factory;
mod progress_updater;
mod status;
mod transfer_status;
mod worker;

use std::{
    collections::HashMap,
    io::SeekFrom,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures::future;
use snafu::ResultExt;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::{mpsc, oneshot, Mutex},
    task::{JoinHandle, JoinSet},
};

pub use self::{
    chunk::{Chunk, MINIMUM_CHUNK_SIZE},
    factory::Factory as DownloaderFactory,
    status::DownloaderStatus,
    transfer_status::TransferStatus,
};
use self::{
    control_file::ControlFile,
    progress_updater::ProgressUpdater,
    worker::{Worker, WorkerEvent},
};
use crate::{error, error::Error, fetcher::Fetcher};

type DownloaderHandle = Option<(mpsc::UnboundedSender<Event>, JoinHandle<Result<Summary, Error>>)>;

pub struct Downloader {
    use_single_worker: bool,
    worker_number: u64,
    transfer_status: TransferStatus,
    sink: File,
    source: Fetcher,
    uri: http::Uri,
    file_path: PathBuf,
    handle: DownloaderHandle,
    is_completed: Arc<AtomicBool>,
}

impl Downloader {
    /// # Errors
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.handle.is_none() && !self.is_completed.load(Ordering::Relaxed) {
            let sink_cloned = self.sink.try_clone().await.with_context(|_| {
                error::CloneFileInstanceSnafu { file_path: self.file_path.clone() }
            })?;
            let (event_sender, event_receiver) = mpsc::unbounded_channel::<Event>();
            let join_handle = if self.use_single_worker {
                tokio::spawn(Self::serve_with_single_worker(
                    self.transfer_status.clone(),
                    sink_cloned,
                    self.source.clone(),
                    self.file_path.clone(),
                    event_receiver,
                    self.is_completed.clone(),
                ))
            } else {
                let (control_file, transfer_status) =
                    ControlFile::new(&self.file_path, self.uri.clone()).await?;
                if let Some(transfer_status) = transfer_status {
                    self.transfer_status = transfer_status;
                }
                tokio::spawn(Self::serve_with_multiple_workers(ServeWithMultipleWorkerOptions {
                    worker_number: self.worker_number,
                    transfer_status: self.transfer_status.clone(),
                    sink: Arc::new(Mutex::new(sink_cloned)),
                    source: self.source.clone(),
                    file_path: self.file_path.clone(),
                    event_sender: event_sender.clone(),
                    event_receiver,
                    control_file,
                    is_completed: self.is_completed.clone(),
                }))
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
    pub async fn join(mut self) -> Result<Option<(TransferStatus, DownloaderStatus)>, Error> {
        if let Some((_event_sender, join_handle)) = self.handle.take() {
            let transfer_status = match join_handle.await.context(error::JoinTaskSnafu)?? {
                Summary::Completed { transfer_status } | Summary::Partial { transfer_status } => {
                    transfer_status
                }
            };
            let mut progress = DownloaderStatus::from(transfer_status.clone());
            progress.set_file_path(&self.file_path);
            Ok(Some((transfer_status, progress)))
        } else {
            Ok(None)
        }
    }

    pub async fn scrape_status(&self) -> Option<DownloaderStatus> {
        if let Some((event_sender, _join_handle)) = self.handle.as_ref() {
            let (send, recv) = oneshot::channel();
            drop(event_sender.send(Event::GetStatus(send)));
            recv.await
                .map(|status| {
                    self.is_completed.store(status.is_completed(), Ordering::Relaxed);
                    let mut progress = DownloaderStatus::from(status);
                    progress.set_file_path(&self.file_path);
                    progress
                })
                .ok()
        } else {
            None
        }
    }

    pub fn is_completed(&self) -> bool { self.is_completed.load(Ordering::Relaxed) }

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

    async fn serve_with_single_worker(
        mut transfer_status: TransferStatus,
        mut sink: File,
        mut source: Fetcher,
        file_path: PathBuf,
        mut event_receiver: mpsc::UnboundedReceiver<Event>,
        is_completed: Arc<AtomicBool>,
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
                    is_completed.store(true, Ordering::Relaxed);
                    transfer_status.mark_chunk_completed(0);
                    summary = Summary::Completed { transfer_status };
                    break;
                }
                future::Either::Left((Err(err), _)) => {
                    tracing::warn!("{err}");
                    break;
                }
                future::Either::Right((Some(Event::GetStatus(sender)), _)) => {
                    drop(sender.send(transfer_status.clone()));
                }
                future::Either::Right((Some(Event::Stop), _)) => {
                    summary = Summary::Partial { transfer_status };
                    break;
                }
                future::Either::Right(_) => {}
            }
        }

        Ok(summary)
    }

    #[allow(clippy::too_many_lines)]
    async fn serve_with_multiple_workers(
        ServeWithMultipleWorkerOptions {
            worker_number,
            mut transfer_status,
            sink,
            source,
            file_path,
            event_sender,
            mut event_receiver,
            mut control_file,
            is_completed,
        }: ServeWithMultipleWorkerOptions,
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
                event_receiver: worker_event_receiver,
            };
            let _handle = join_set.spawn(worker.serve());
        }

        for chunk in transfer_status.chunks() {
            drop(chunk_sender.send(chunk).await);
        }
        transfer_status.update_concurrent_number(worker_event_senders.len());

        let mut next_worker_id = worker_number;
        let mut chunk_to_worker = HashMap::new();

        let mut summary = Summary::Partial { transfer_status: transfer_status.clone() };
        while let Some(event) = event_receiver.recv().await {
            match event {
                Event::ChunkTransferStarted { worker_id, chunk_start } => {
                    let _unused = chunk_to_worker.insert(chunk_start, worker_id);
                    transfer_status.update_concurrent_number(worker_event_senders.len());
                }
                Event::ChunkTransferCompleted { chunk_start, worker_id: _worker_id } => {
                    let _unused = chunk_to_worker.remove(&chunk_start);
                    transfer_status.mark_chunk_completed(chunk_start);

                    if transfer_status.is_completed() {
                        is_completed.store(true, Ordering::Relaxed);
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
                Event::GetStatus(sender) => drop(sender.send(transfer_status.clone())),
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
                            event_receiver: worker_event_receiver,
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
                    transfer_status.update_concurrent_number(worker_event_senders.len());
                }
                Event::RemoveWorker => {
                    if let Some((origin_chunk, new_chunk)) = transfer_status.freeze() {
                        if let Some(worker_id) = chunk_to_worker.get(&origin_chunk.start) {
                            if let Some(worker) = worker_event_senders.remove(worker_id) {
                                let (sender, wait) = oneshot::channel();
                                drop(worker.send(WorkerEvent::Remove(sender)));
                                let _ = wait.await;
                            }
                        }
                        let _ = chunk_sender.send(new_chunk).await;
                    }
                    transfer_status.update_concurrent_number(worker_event_senders.len());
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

struct ServeWithMultipleWorkerOptions {
    worker_number: u64,
    transfer_status: TransferStatus,
    sink: Arc<Mutex<File>>,
    source: Fetcher,
    file_path: PathBuf,
    event_sender: mpsc::UnboundedSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
    control_file: ControlFile,
    is_completed: Arc<AtomicBool>,
}

enum Event {
    Stop,
    GetStatus(oneshot::Sender<TransferStatus>),
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
