use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{task_scheduler::TaskStatus, NewTask};

#[derive(Debug)]
pub enum Event {
    Shutdown,
    TryStartTask,
    CheckProgress,
    AddUri { new_task: NewTask, start_immediately: bool },
    RemoveTask { task_id: Uuid },
    PauseTask { task_id: Uuid },
    ResumeTask { task_id: Uuid },
    GetTaskStatus { task_id: Uuid, sender: oneshot::Sender<TaskStatus> },
    TaskCompleted { task_id: Uuid },
    IncreaseWorkerNumber { task_id: Uuid },
    DecreaseWorkerNumber { task_id: Uuid },
}
