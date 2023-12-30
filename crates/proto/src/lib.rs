mod error;
mod utils;
mod proto {
    // SAFETY: allow: prost
    #![allow(
        box_pointers,
        unreachable_pub,
        unused_qualifications,
        unused_results,
        clippy::default_trait_access,
        clippy::derive_partial_eq_without_eq,
        clippy::doc_markdown,
        clippy::future_not_send,
        clippy::large_enum_variant,
        clippy::missing_const_for_fn,
        clippy::missing_errors_doc,
        clippy::must_use_candidate,
        clippy::return_self_not_must_use,
        clippy::similar_names,
        clippy::too_many_lines,
        clippy::trivially_copy_pass_by_ref,
        clippy::use_self,
        clippy::wildcard_imports
    )]

    tonic::include_proto!("caracal");
}

use caracal_base::model;

pub use self::{
    proto::{
        system_client::SystemClient,
        system_server::{System, SystemServer},
        task_client::TaskClient,
        task_server::{Task, TaskServer},
        AddUriRequest, AddUriResponse, Chunk, DecreaseConcurrentNumberRequest,
        DecreaseConcurrentNumberResponse, GetAllTaskStatusesResponse, GetSystemVersionResponse,
        GetTaskStatusRequest, GetTaskStatusResponse, IncreaseConcurrentNumberRequest,
        IncreaseConcurrentNumberResponse, PauseAllTasksResponse, PauseTaskRequest,
        PauseTaskResponse, Priority, RemoveTaskRequest, RemoveTaskResponse, ResumeAllTasksResponse,
        ResumeTaskRequest, ResumeTaskResponse, TaskMetadata, TaskState, TaskStatus,
    },
    utils::{datetime_to_timestamp, timestamp_to_datetime},
};

impl From<Priority> for model::Priority {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Lowest => Self::Lowest,
            Priority::Low => Self::Low,
            Priority::Normal => Self::Normal,
            Priority::High => Self::High,
            Priority::Highest => Self::Highest,
        }
    }
}

impl From<model::Priority> for Priority {
    fn from(value: model::Priority) -> Self {
        match value {
            model::Priority::Lowest => Self::Lowest,
            model::Priority::Low => Self::Low,
            model::Priority::Normal => Self::Normal,
            model::Priority::High => Self::High,
            model::Priority::Highest => Self::Highest,
        }
    }
}

impl From<TaskState> for model::TaskState {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::Pending => Self::Pending,
            TaskState::Downloading => Self::Downloading,
            TaskState::Paused => Self::Paused,
            TaskState::Completed => Self::Completed,
            TaskState::Canceled => Self::Canceled,
            TaskState::Failed => Self::Failed,
        }
    }
}

impl From<model::TaskState> for TaskState {
    fn from(value: model::TaskState) -> Self {
        match value {
            model::TaskState::Pending => Self::Pending,
            model::TaskState::Downloading => Self::Downloading,
            model::TaskState::Paused => Self::Paused,
            model::TaskState::Completed => Self::Completed,
            model::TaskState::Canceled => Self::Canceled,
            model::TaskState::Failed => Self::Failed,
        }
    }
}

impl From<model::ProgressChunk> for Chunk {
    fn from(
        model::ProgressChunk { start, end, received, is_completed }: model::ProgressChunk,
    ) -> Self {
        Self { start, end, received, is_completed }
    }
}

impl From<Chunk> for model::ProgressChunk {
    fn from(Chunk { start, end, received, is_completed }: Chunk) -> Self {
        Self { start, end, received, is_completed }
    }
}
