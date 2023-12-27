mod error;
mod event;
mod worker;

use snafu::OptionExt;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use uuid::Uuid;

pub use self::error::{Error, Result};
use self::{event::Event, worker::Worker};
use crate::{downloader::DownloaderFactory, DownloaderStatus, NewTask};

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
    pub fn add_uri(&self, new_task: NewTask, start_immediately: bool) -> Result<()> {
        if self.event_sender.send(Event::AddUri { new_task, start_immediately }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub fn pause_task(&self, task_id: Uuid) -> Result<()> {
        if self.event_sender.send(Event::PauseTask { task_id }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub fn resume_task(&self, task_id: Uuid) -> Result<()> {
        if self.event_sender.send(Event::ResumeTask { task_id }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub fn remove_task(&self, task_id: Uuid) -> Result<()> {
        if self.event_sender.send(Event::RemoveTask { task_id }).is_err() {
            return Err(Error::TaskSchedulerClosed);
        }
        Ok(())
    }

    /// # Errors
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<TaskStatus> {
        let (sender, receiver) = oneshot::channel();
        if self.event_sender.send(Event::GetTaskStatus { task_id, sender }).is_err() {
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

#[derive(Clone, Copy, Debug)]
pub enum TaskState {
    Pending,
    Downloading,
    Paused,
    Canceled,
    Completed,
}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub status: DownloaderStatus,

    pub state: TaskState,
}
