use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    time::Duration,
};

use caracal_base::model;
use futures::{future, FutureExt};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::{
    downloader::DownloaderFactory, ext::UriExt, task_scheduler::Event, Downloader, DownloaderStatus,
};

#[derive(Debug)]
pub struct Worker {
    pub factory: DownloaderFactory,

    pub event_sender: mpsc::UnboundedSender<Event>,

    pub event_receiver: mpsc::UnboundedReceiver<Event>,

    pub max_concurrent_task_number: usize,
}

impl Worker {
    pub async fn serve(self) {
        let Self { factory, event_sender, mut event_receiver, max_concurrent_task_number } = self;

        let mut event_handler = EventHandler {
            factory,
            event_sender: event_sender.clone(),
            max_concurrent_task_number,
            tasks: HashMap::new(),
            pending_tasks: BinaryHeap::new(),
            downloaders: HashMap::new(),
            completed_tasks: Vec::new(),
            paused_tasks: HashSet::new(),
            canceled_tasks: HashSet::new(),
            download_progresses: HashMap::new(),
        };

        let timer = tokio::spawn({
            async move {
                let mut interval = tokio::time::interval(Duration::from_millis(200));
                loop {
                    let _ = interval.tick().await;
                    if event_sender.send(Event::CheckProgress).is_err() {
                        break;
                    }
                }
            }
        });

        while let Some(event) = event_receiver.recv().await {
            match event {
                Event::CheckProgress => {
                    event_handler.check_progress().await;
                }
                Event::Shutdown => {
                    event_handler.on_shutdown().await;
                    break;
                }
                Event::TryStartTask => {
                    event_handler.try_start_task().await;
                }
                Event::AddUri { new_task, start_immediately, sender } => {
                    event_handler.add_uri(new_task, start_immediately, sender);
                }
                Event::RemoveTask { task_id, sender } => {
                    event_handler.remove_task(task_id, sender).await;
                }
                Event::PauseTask { task_id, sender } => {
                    event_handler.pause_task(task_id, sender).await;
                }
                Event::PauseAllTasks => {
                    event_handler.pause_all_tasks().await;
                }
                Event::ResumeTask { task_id, sender } => {
                    event_handler.resume_task(task_id, sender);
                }
                Event::ResumeAllTasks => {
                    event_handler.resume_all_tasks();
                }
                Event::GetTaskStatus { task_id, sender } => {
                    event_handler.get_task_status(task_id, sender);
                }
                Event::GetAllTasks { sender } => {
                    event_handler.get_all_tasks(sender);
                }
                Event::GetPendingTasks { sender } => {
                    event_handler.get_pending_tasks(sender);
                }
                Event::GetDownloadingTasks { sender } => {
                    event_handler.get_downloading_tasks(sender);
                }
                Event::GetPausedTasks { sender } => {
                    event_handler.get_paused_tasks(sender);
                }
                Event::GetCompletedTasks { sender } => {
                    event_handler.get_completed_tasks(sender);
                }
                Event::GetCanceledTasks { sender } => {
                    event_handler.get_canceled_tasks(sender);
                }
                Event::GetAllTaskStatuses { sender } => {
                    event_handler.get_all_task_statuses(sender);
                }
                Event::TaskCompleted { task_id } => {
                    event_handler.on_task_completed(task_id).await;
                }
                Event::IncreaseWorkerNumber { task_id } => {
                    event_handler.increase_worker_number(task_id);
                }
                Event::DecreaseWorkerNumber { task_id } => {
                    event_handler.decrease_worker_number(task_id);
                }
            }
        }

        // close event receiver
        drop(event_receiver);

        // we do not care the result, drop it.
        drop(timer.await);
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PendingTask {
    pub priority: model::Priority,

    pub timestamp: Reverse<time::OffsetDateTime>,

    pub task_id: Uuid,
}

struct EventHandler {
    factory: DownloaderFactory,
    event_sender: mpsc::UnboundedSender<Event>,
    max_concurrent_task_number: usize,
    tasks: HashMap<Uuid, model::CreateTask>,
    pending_tasks: BinaryHeap<PendingTask>,
    downloaders: HashMap<Uuid, Downloader>,
    completed_tasks: Vec<Uuid>,
    paused_tasks: HashSet<Uuid>,
    canceled_tasks: HashSet<Uuid>,
    download_progresses: HashMap<Uuid, DownloaderStatus>,
}

impl EventHandler {
    async fn check_progress(&mut self) {
        let futs = self.downloaders.iter().map(|(&task_id, downloader)| {
            async move { (task_id, downloader.is_completed(), downloader.scrape_status().await) }.boxed()
        });
        let maybe_progresses = future::join_all(futs).await;
        for (task_id, is_completed, maybe_progress) in maybe_progresses {
            if let Some(progress) = maybe_progress {
                drop(self.download_progresses.insert(task_id, progress));
            }
            if is_completed {
                drop(self.event_sender.send(Event::TaskCompleted { task_id }));
            }
        }
    }

    async fn on_shutdown(mut self) {
        self.pending_tasks.clear();

        // stop all downloading tasks
        let futs = self.downloaders.drain().map(|(task_id, mut downloader)| {
            async move {
                tracing::info!("Stopping task {task_id}");
                if let Err(err) = downloader.pause().await {
                    tracing::error!("{err}");
                }
                if let Err(err) = downloader.join().await {
                    tracing::error!("{err}");
                }
                tracing::info!("Stopped task {task_id}");
            }
            .boxed()
        });
        let _results = future::join_all(futs).await;
    }

    async fn try_start_task(&mut self) {
        if self.downloaders.len() < self.max_concurrent_task_number {
            // start downloader
            let task_id = if let Some(task) = self.pending_tasks.pop() {
                task.task_id
            } else {
                tracing::debug!("No pending tasks");
                return;
            };

            let new_task = self.tasks.get(&task_id).expect("task must exist");
            tracing::info!("Starting task {task_id}, URI: {uri}", uri = new_task.uri);

            match self.factory.create_new_task(new_task).await {
                Ok(mut downloader) => {
                    if let Err(err) = downloader.start().await {
                        tracing::error!("{err}");
                        self.completed_tasks.push(task_id);
                        if let Some(progress) = downloader.scrape_status().await {
                            drop(self.download_progresses.insert(task_id, progress));
                        }
                    } else {
                        tracing::info!("Started task {task_id}, URI: {uri}", uri = new_task.uri);
                        drop(self.downloaders.insert(task_id, downloader));
                    }
                }
                Err(err) => {
                    tracing::error!("{err}");
                    self.completed_tasks.push(task_id);
                }
            }
        }
    }

    fn add_uri(
        &mut self,
        new_task: model::CreateTask,
        start_immediately: bool,
        sender: oneshot::Sender<Uuid>,
    ) {
        let (task_id, priority, timestamp) =
            (Uuid::new_v4(), new_task.priority, Reverse(new_task.creation_timestamp));

        tracing::info!(
            "Added new task {task_id}, URI: {uri}, start immediately: {start_immediately}",
            uri = new_task.uri
        );
        drop(
            self.download_progresses
                .insert(task_id, DownloaderStatus::with_file_path(new_task.uri.guess_filename())),
        );
        drop(self.tasks.insert(task_id, new_task));

        if start_immediately {
            self.pending_tasks.push(PendingTask { priority, timestamp, task_id });
        } else {
            let _ = self.paused_tasks.insert(task_id);
        }
        drop(self.event_sender.send(Event::TryStartTask));
        let _ = sender.send(task_id);
    }

    async fn remove_task(&mut self, task_id: Uuid, sender: oneshot::Sender<Option<Uuid>>) {
        let task_id = if let Some(mut downloader) = self.downloaders.remove(&task_id) {
            tracing::info!("Removing task {task_id}");

            if let Err(err) = downloader.pause().await {
                tracing::error!("{err}");
            }
            if let Err(err) = downloader.join().await {
                tracing::error!("{err}");
            }
            tracing::info!("Removed task {task_id}");

            let _ = self.canceled_tasks.insert(task_id);
            Some(task_id)
        } else {
            None
        };
        let _ = sender.send(task_id);
    }

    async fn pause_task(&mut self, task_id: Uuid, sender: oneshot::Sender<Option<Uuid>>) {
        let task_id = if let Some(mut downloader) = self.downloaders.remove(&task_id) {
            tracing::info!("Pausing task {task_id}");

            if let Err(err) = downloader.pause().await {
                tracing::error!("{err}");
            }
            if let Err(err) = downloader.join().await {
                tracing::error!("{err}");
            }
            tracing::info!("Paused task {task_id}");

            let _ = self.paused_tasks.insert(task_id);
            drop(self.event_sender.send(Event::TryStartTask));
            Some(task_id)
        } else {
            None
        };
        let _ = sender.send(task_id);
    }

    async fn pause_all_tasks(&mut self) {
        tracing::info!("Pausing all tasks");
        let mut futs = Vec::new();
        for (task_id, mut downloader) in self.downloaders.drain() {
            let _ = self.paused_tasks.insert(task_id);
            futs.push(
                async move {
                    if let Err(err) = downloader.pause().await {
                        tracing::error!("{err}");
                    }
                    if let Err(err) = downloader.join().await {
                        tracing::error!("{err}");
                    }
                }
                .boxed(),
            );
        }
        drop(future::join_all(futs).await);
        tracing::info!("Paused all tasks");
    }

    fn resume_task(&mut self, task_id: Uuid, sender: oneshot::Sender<Option<Uuid>>) {
        tracing::info!("Resuming task {task_id}");
        let task_id = if self.paused_tasks.remove(&task_id) {
            let model::CreateTask { priority, creation_timestamp, .. } =
                self.tasks.get(&task_id).expect("task must exist");
            self.pending_tasks.push(PendingTask {
                priority: *priority,
                timestamp: Reverse(*creation_timestamp),
                task_id,
            });
            drop(self.event_sender.send(Event::TryStartTask));
            Some(task_id)
        } else {
            None
        };
        let _ = sender.send(task_id);
    }

    fn resume_all_tasks(&mut self) {
        tracing::info!("Resuming all tasks");
        for task_id in self.paused_tasks.drain() {
            let model::CreateTask { priority, creation_timestamp, .. } =
                self.tasks.get(&task_id).expect("task must exist");
            self.pending_tasks.push(PendingTask {
                priority: *priority,
                timestamp: Reverse(*creation_timestamp),
                task_id,
            });
            drop(self.event_sender.send(Event::TryStartTask));
        }
    }

    #[inline]
    fn get_all_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.tasks.keys().copied().collect()));
    }

    #[inline]
    fn get_pending_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.pending_tasks.iter().map(|task| task.task_id).collect()));
    }

    #[inline]
    fn get_downloading_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.downloaders.keys().copied().collect()));
    }

    #[inline]
    fn get_canceled_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.canceled_tasks.iter().copied().collect()));
    }

    #[inline]
    fn get_paused_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.paused_tasks.iter().copied().collect()));
    }

    #[inline]
    fn get_completed_tasks(&self, sender: oneshot::Sender<Vec<Uuid>>) {
        drop(sender.send(self.completed_tasks.clone()));
    }

    fn get_task_status_inner(&self, task_id: &Uuid) -> Option<model::TaskStatus> {
        self.tasks.get(task_id).and_then(|task| {
            let state = if self.canceled_tasks.contains(task_id) {
                model::TaskState::Canceled
            } else if self.downloaders.contains_key(task_id) {
                model::TaskState::Downloading
            } else if self.completed_tasks.contains(task_id) {
                model::TaskState::Completed
            } else if self.paused_tasks.contains(task_id) {
                model::TaskState::Paused
            } else {
                model::TaskState::Pending
            };

            self.download_progresses.get(task_id).cloned().map(
                |downloader_status: DownloaderStatus| model::TaskStatus {
                    id: *task_id,
                    content_length: downloader_status.content_length(),
                    chunks: downloader_status.chunks(),
                    concurrent_number: downloader_status.concurrent_number(),
                    file_path: downloader_status.file_path().to_path_buf(),
                    state,
                    priority: task.priority,
                    creation_timestamp: task.creation_timestamp,
                },
            )
        })
    }

    #[inline]
    fn get_task_status(&self, task_id: Uuid, sender: oneshot::Sender<Option<model::TaskStatus>>) {
        drop(sender.send(self.get_task_status_inner(&task_id)));
    }

    #[inline]
    fn get_all_task_statuses(&self, sender: oneshot::Sender<Vec<model::TaskStatus>>) {
        let task_statuses =
            self.tasks.keys().filter_map(|id| self.get_task_status_inner(id)).collect();
        drop(sender.send(task_statuses));
    }

    async fn on_task_completed(&mut self, task_id: Uuid) {
        tracing::info!("Completed task {task_id}");
        self.completed_tasks.push(task_id);
        if let Some(downloader) = self.downloaders.remove(&task_id) {
            match downloader.join().await {
                Ok(Some((_, downloader_status))) => {
                    drop(self.download_progresses.insert(task_id, downloader_status));
                }
                Ok(None) => {}
                Err(err) => {
                    tracing::error!("{err}");
                }
            }
        }
        drop(self.event_sender.send(Event::TryStartTask));
    }

    #[inline]
    fn increase_worker_number(&mut self, task_id: Uuid) {
        if let Some(downloader) = self.downloaders.get(&task_id) {
            downloader.add_worker();
        }
    }

    #[inline]
    fn decrease_worker_number(&mut self, task_id: Uuid) {
        if let Some(downloader) = self.downloaders.get(&task_id) {
            downloader.remove_worker();
        }
    }
}
