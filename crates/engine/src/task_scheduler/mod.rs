mod error;
mod event;
mod worker;

use caracal_base::model;
use snafu::OptionExt;
use time::OffsetDateTime;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use uuid::Uuid;

pub use self::error::{Error, Result};
use self::{event::Event, worker::Worker};
use crate::{downloader::DownloaderFactory, DownloaderStatus};

#[derive(Clone, Debug)]
pub struct TaskScheduler {
    event_sender: mpsc::UnboundedSender<Event>,
}

impl TaskScheduler {
    #[must_use]
    pub fn new(
        factory: DownloaderFactory,
        max_concurrent_task_number: usize,
    ) -> (Self, JoinHandle<()>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let join_handle = tokio::spawn({
            let event_sender = event_sender.clone();
            async move {
                Worker { factory, event_sender, event_receiver, max_concurrent_task_number }
                    .serve()
                    .await;
            }
        });
        (Self { event_sender }, join_handle)
    }

    /// # Errors
    pub async fn add_uri(
        &self,
        new_task: model::CreateTask,
        start_immediately: bool,
    ) -> Result<Uuid> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::AddUri { new_task, start_immediately, sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }

        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn pause_task(&self, task_id: Uuid) -> Result<Option<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::PauseTask { task_id, sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub fn pause_all_tasks(&self) -> Result<()> {
        if self.event_sender.send(Event::PauseAllTasks).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub async fn resume_task(&self, task_id: Uuid) -> Result<Option<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::ResumeTask { task_id, sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub fn resume_all_tasks(&self) -> Result<()> {
        if self.event_sender.send(Event::ResumeAllTasks).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub async fn remove_task(&self, task_id: Uuid) -> Result<Option<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::RemoveTask { task_id, sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<Option<TaskStatus>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetTaskStatus { task_id, sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_all_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetAllTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_pending_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetPendingTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_downloading_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetDownloadingTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_paused_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetPausedTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_canceled_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetCanceledTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub async fn get_completed_tasks(&self) -> Result<Vec<Uuid>> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetCompletedTasks { sender }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        receiver.await.ok().context(error::TaskSchedulerClosedSnafu)
    }

    /// # Errors
    pub fn shutdown(&self) -> Result<()> {
        if self.event_sender.send(Event::Shutdown).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub fn increase_concurrent_number(&self, task_id: Uuid) -> Result<()> {
        if self.event_sender.send(Event::IncreaseWorkerNumber { task_id }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub fn decrease_concurrent_number(&self, task_id: Uuid) -> Result<()> {
        if self.event_sender.send(Event::DecreaseWorkerNumber { task_id }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub id: Uuid,

    pub status: DownloaderStatus,

    pub state: model::TaskState,

    pub priority: model::Priority,

    pub creation_timestamp: OffsetDateTime,
}
