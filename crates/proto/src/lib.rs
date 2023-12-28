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

use snafu::ResultExt;

pub use self::{
    proto::{
        system_client::SystemClient,
        system_server::{System, SystemServer},
        task_client::TaskClient,
        task_server::{Task, TaskServer},
        AddUriRequest, AddUriResponse, Chunk, GetSystemVersionResponse, GetTaskStatusRequest,
        GetTaskStatusResponse, PauseAllTasksResponse, PauseTaskRequest, PauseTaskResponse,
        Priority, RemoveTaskRequest, RemoveTaskResponse, ResumeAllTasksResponse, ResumeTaskRequest,
        ResumeTaskResponse, TaskMetadata, TaskState, Uuid,
    },
    utils::{datetime_to_timestamp, timestamp_to_datetime},
};

impl From<Priority> for caracal_base::Priority {
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

impl From<caracal_base::Priority> for Priority {
    fn from(value: caracal_base::Priority) -> Self {
        match value {
            caracal_base::Priority::Lowest => Self::Lowest,
            caracal_base::Priority::Low => Self::Low,
            caracal_base::Priority::Normal => Self::Normal,
            caracal_base::Priority::High => Self::High,
            caracal_base::Priority::Highest => Self::Highest,
        }
    }
}

impl From<TaskState> for caracal_base::TaskState {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::Pending => Self::Pending,
            TaskState::Downloading => Self::Downloading,
            TaskState::Paused => Self::Paused,
            TaskState::Completed => Self::Completed,
            TaskState::Canceled => Self::Canceled,
        }
    }
}

impl From<caracal_base::TaskState> for TaskState {
    fn from(value: caracal_base::TaskState) -> Self {
        match value {
            caracal_base::TaskState::Pending => Self::Pending,
            caracal_base::TaskState::Downloading => Self::Downloading,
            caracal_base::TaskState::Paused => Self::Paused,
            caracal_base::TaskState::Completed => Self::Completed,
            caracal_base::TaskState::Canceled => Self::Canceled,
        }
    }
}

impl TryFrom<Uuid> for uuid::Uuid {
    type Error = error::UnexpectedDataFormatError;

    #[inline]
    fn try_from(Uuid { ref value }: Uuid) -> Result<Self, Self::Error> {
        Self::from_slice(value).map_err(Box::from).with_context(|_| error::UnexpectedFormatSnafu {})
    }
}

impl From<uuid::Uuid> for Uuid {
    #[inline]
    fn from(uuid: uuid::Uuid) -> Self { Self { value: uuid.as_u128().to_be_bytes().to_vec() } }
}
