use caracal_base::model;

// FIXME: use `AddTask`
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Action {
    GetAllTaskStatuses,
    SelectTask { task_id: u64 },
    // TODO: use it
    AddTask(model::CreateTask),
    RemoveTask { task_id: u64 },
    PauseTask { task_id: u64 },
    ResumeTask { task_id: u64 },
    IncreaseConcurrentNumber { task_id: u64 },
    DecreaseConcurrentNumber { task_id: u64 },
    Shutdown,
}
