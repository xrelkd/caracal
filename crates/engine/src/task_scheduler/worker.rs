use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    time::Duration,
};

use caracal_base::model;
use futures::{future, FutureExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    downloader::DownloaderFactory,
    task_scheduler::{Event, TaskStatus},
    Downloader,
};

#[derive(Debug)]
pub struct Worker {
    pub factory: DownloaderFactory,

    pub event_sender: mpsc::UnboundedSender<Event>,

    pub event_receiver: mpsc::UnboundedReceiver<Event>,

    pub max_concurrent_task_number: usize,
}

impl Worker {
    // FIXME: split this function
    #[allow(clippy::too_many_lines)]
    pub async fn serve(self) {
        let Self { factory, event_sender, mut event_receiver, max_concurrent_task_number } = self;
        let mut tasks = HashMap::new();
        let mut pending_tasks = BinaryHeap::<PendingTask>::new();
        let mut downloading_tasks = HashMap::new();
        let mut completed_tasks = Vec::new();
        let mut paused_tasks = HashSet::new();
        let mut canceled_tasks = HashSet::new();
        let mut download_progresses = HashMap::new();
        let timer = tokio::spawn({
            let event_sender = event_sender.clone();
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
                    let futs =
                        downloading_tasks.iter().map(|(&task_id, downloader): (_, &Downloader)| {
                            async move { (task_id, downloader.scrape_status().await) }.boxed()
                        });
                    let maybe_progresses = future::join_all(futs).await;
                    for (task_id, maybe_progress) in maybe_progresses {
                        if let Some(progress) = maybe_progress {
                            let is_completed = progress.is_completed();
                            drop(download_progresses.insert(task_id, progress));
                            if is_completed {
                                drop(event_sender.send(Event::TaskCompleted { task_id }));
                            }
                        }
                    }
                }
                Event::Shutdown => {
                    pending_tasks.clear();

                    // stop all downloading tasks
                    let futs = downloading_tasks.drain().map(|(_task_id, mut downloader)| {
                        async move {
                            if let Err(err) = downloader.pause().await {
                                tracing::error!("{err}");
                            }
                            if let Err(err) = downloader.join().await {
                                tracing::error!("{err}");
                            }
                        }
                        .boxed()
                    });
                    let _results = future::join_all(futs).await;

                    break;
                }
                Event::TryStartTask => {
                    if downloading_tasks.len() < max_concurrent_task_number {
                        // start downloader
                        let task_id = if let Some(task) = pending_tasks.pop() {
                            task.task_id
                        } else {
                            tracing::info!("No pending tasks");
                            continue;
                        };

                        tracing::info!("Try to start new task {task_id}");
                        let new_task = tasks.get(&task_id).expect("task must exist");
                        match factory.create_new_task(new_task).await {
                            Ok(mut downloader) => {
                                if let Err(err) = downloader.start().await {
                                    tracing::error!("{err}");
                                    completed_tasks.push(task_id);
                                    if let Some(progress) = downloader.scrape_status().await {
                                        drop(download_progresses.insert(task_id, progress));
                                    }
                                } else {
                                    drop(downloading_tasks.insert(task_id, downloader));
                                }
                            }
                            Err(err) => {
                                tracing::error!("{err}");
                                completed_tasks.push(task_id);
                            }
                        }
                    }
                }
                Event::AddUri { new_task, start_immediately, sender } => {
                    let (task_id, priority, timestamp) =
                        (Uuid::new_v4(), new_task.priority, Reverse(new_task.creation_timestamp));
                    drop(tasks.insert(task_id, new_task));
                    if start_immediately {
                        pending_tasks.push(PendingTask { priority, timestamp, task_id });
                    } else {
                        let _ = paused_tasks.insert(task_id);
                    }
                    drop(event_sender.send(Event::TryStartTask));
                    let _ = sender.send(task_id);
                }
                Event::RemoveTask { task_id, sender } => {
                    let task_id = if let Some(mut downloader) = downloading_tasks.remove(&task_id) {
                        if let Err(err) = downloader.pause().await {
                            tracing::error!("{err}");
                        }
                        if let Err(err) = downloader.join().await {
                            tracing::error!("{err}");
                        }
                        let _ = canceled_tasks.insert(task_id);
                        Some(task_id)
                    } else {
                        None
                    };
                    let _ = sender.send(task_id);
                }
                Event::PauseTask { task_id, sender } => {
                    let task_id = if let Some(mut downloader) = downloading_tasks.remove(&task_id) {
                        if let Err(err) = downloader.pause().await {
                            tracing::error!("{err}");
                        }
                        if let Err(err) = downloader.join().await {
                            tracing::error!("{err}");
                        }

                        let _ = paused_tasks.insert(task_id);
                        Some(task_id)
                    } else {
                        None
                    };
                    let _ = sender.send(task_id);
                }
                Event::PauseAllTasks => {
                    let mut futs = Vec::new();
                    for (task_id, mut downloader) in downloading_tasks.drain() {
                        let _ = paused_tasks.insert(task_id);
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
                }
                Event::ResumeTask { task_id, sender } => {
                    let task_id = if paused_tasks.remove(&task_id) {
                        let model::CreateTask { priority, creation_timestamp, .. } =
                            tasks.get(&task_id).expect("task must exist");
                        pending_tasks.push(PendingTask {
                            priority: *priority,
                            timestamp: Reverse(*creation_timestamp),
                            task_id,
                        });
                        drop(event_sender.send(Event::TryStartTask));
                        Some(task_id)
                    } else {
                        None
                    };
                    let _ = sender.send(task_id);
                }
                Event::ResumeAllTasks => {
                    for task_id in paused_tasks.drain() {
                        let model::CreateTask { priority, creation_timestamp, .. } =
                            tasks.get(&task_id).expect("task must exist");
                        pending_tasks.push(PendingTask {
                            priority: *priority,
                            timestamp: Reverse(*creation_timestamp),
                            task_id,
                        });
                        drop(event_sender.send(Event::TryStartTask));
                    }
                }
                Event::GetTaskStatus { task_id, sender } => {
                    let task_status = tasks.get(&task_id).and_then(|task| {
                        let state = if canceled_tasks.contains(&task_id) {
                            model::TaskState::Canceled
                        } else if downloading_tasks.contains_key(&task_id) {
                            model::TaskState::Downloading
                        } else if completed_tasks.contains(&task_id) {
                            model::TaskState::Completed
                        } else if paused_tasks.contains(&task_id) {
                            model::TaskState::Paused
                        } else {
                            model::TaskState::Pending
                        };
                        download_progresses.get(&task_id).cloned().map(|status| TaskStatus {
                            id: task_id,
                            status,
                            state,
                            priority: task.priority,
                            creation_timestamp: task.creation_timestamp,
                        })
                    });

                    drop(sender.send(task_status));
                }
                Event::GetAllTasks { sender } => {
                    drop(sender.send(tasks.keys().copied().collect()));
                }
                Event::GetPendingTasks { sender } => {
                    drop(sender.send(pending_tasks.iter().map(|task| task.task_id).collect()));
                }
                Event::GetDownloadingTasks { sender } => {
                    drop(sender.send(downloading_tasks.keys().copied().collect()));
                }
                Event::GetPausedTasks { sender } => {
                    drop(sender.send(paused_tasks.iter().copied().collect()));
                }
                Event::GetCompletedTasks { sender } => {
                    drop(sender.send(completed_tasks.clone()));
                }
                Event::GetCanceledTasks { sender } => {
                    drop(sender.send(canceled_tasks.iter().copied().collect()));
                }
                Event::TaskCompleted { task_id } => {
                    tracing::info!("Task {task_id} is completed");
                    completed_tasks.push(task_id);
                    if let Some(downloader) = downloading_tasks.remove(&task_id) {
                        if let Err(err) = downloader.join().await {
                            tracing::error!("{err}");
                        }
                    }
                    drop(event_sender.send(Event::TryStartTask));
                }
                Event::IncreaseWorkerNumber { task_id } => {
                    if let Some(downloader) = downloading_tasks.remove(&task_id) {
                        downloader.add_worker();
                    }
                }
                Event::DecreaseWorkerNumber { task_id } => {
                    if let Some(downloader) = downloading_tasks.remove(&task_id) {
                        downloader.remove_worker();
                    }
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
