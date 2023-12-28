use caracal_base::model;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::task_scheduler::TaskStatus;

#[derive(Debug)]
pub enum Event {
    Shutdown,
    TryStartTask,
    CheckProgress,
    AddUri { new_task: model::CreateTask, start_immediately: bool, sender: oneshot::Sender<Uuid> },
    RemoveTask { task_id: Uuid, sender: oneshot::Sender<Option<Uuid>> },
    PauseTask { task_id: Uuid, sender: oneshot::Sender<Option<Uuid>> },
    PauseAllTasks,
    ResumeTask { task_id: Uuid, sender: oneshot::Sender<Option<Uuid>> },
    ResumeAllTasks,
    GetAllTasks { sender: oneshot::Sender<Vec<Uuid>> },
    GetTaskStatus { task_id: Uuid, sender: oneshot::Sender<Option<TaskStatus>> },
    GetPendingTasks { sender: oneshot::Sender<Vec<Uuid>> },
    GetDownloadingTasks { sender: oneshot::Sender<Vec<Uuid>> },
    GetPausedTasks { sender: oneshot::Sender<Vec<Uuid>> },
    GetCompletedTasks { sender: oneshot::Sender<Vec<Uuid>> },
    GetCanceledTasks { sender: oneshot::Sender<Vec<Uuid>> },
    TaskCompleted { task_id: Uuid },
    IncreaseWorkerNumber { task_id: Uuid },
    DecreaseWorkerNumber { task_id: Uuid },
}
