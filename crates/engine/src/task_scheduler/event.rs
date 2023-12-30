use caracal_base::model;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Event {
    Shutdown,
    TryStartTask,
    CheckProgress,
    AddUri { new_task: model::CreateTask, start_immediately: bool, sender: oneshot::Sender<u64> },
    RemoveTask { task_id: u64, sender: oneshot::Sender<Option<u64>> },
    PauseTask { task_id: u64, sender: oneshot::Sender<Option<u64>> },
    PauseAllTasks,
    ResumeTask { task_id: u64, sender: oneshot::Sender<Option<u64>> },
    ResumeAllTasks,
    GetAllTasks { sender: oneshot::Sender<Vec<u64>> },
    GetTaskStatus { task_id: u64, sender: oneshot::Sender<Option<model::TaskStatus>> },
    GetAllTaskStatuses { sender: oneshot::Sender<Vec<model::TaskStatus>> },
    GetPendingTasks { sender: oneshot::Sender<Vec<u64>> },
    GetDownloadingTasks { sender: oneshot::Sender<Vec<u64>> },
    GetPausedTasks { sender: oneshot::Sender<Vec<u64>> },
    GetCompletedTasks { sender: oneshot::Sender<Vec<u64>> },
    GetCanceledTasks { sender: oneshot::Sender<Vec<u64>> },
    TaskCompleted { task_id: u64 },
    IncreaseConcurrentNumber { task_id: u64 },
    DecreaseConcurrentNumber { task_id: u64 },
}
