use std::{fmt, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::Priority;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskState {
    Pending,
    Downloading,
    Paused,
    Canceled,
    Completed,
    Failed,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "Pending",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Canceled => "Canceled",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateTask {
    #[serde(with = "crate::serde::uri")]
    pub uri: http::Uri,

    pub filename: Option<PathBuf>,

    pub output_directory: PathBuf,

    pub concurrent_number: Option<u64>,

    pub connection_timeout: Option<Duration>,

    pub priority: Priority,

    pub creation_timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskStatus {
    pub id: Uuid,

    pub file_path: PathBuf,

    pub content_length: u64,

    pub chunks: Vec<ProgressChunk>,

    pub concurrent_number: usize,

    pub state: TaskState,

    pub priority: Priority,

    pub creation_timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProgressChunk {
    pub start: u64,

    pub end: u64,

    pub received: u64,

    pub is_completed: bool,
}

// SAFETY: we do not need to record empty progress chunk
#[allow(clippy::len_without_is_empty)]
impl ProgressChunk {
    #[must_use]
    pub const fn len(&self) -> u64 { self.end - self.start + 1 }

    #[must_use]
    pub const fn remaining(&self) -> u64 {
        let len = self.len();
        if len >= self.received {
            len - self.received
        } else {
            0
        }
    }

    #[must_use]
    pub const fn is_completed(&self) -> bool { self.is_completed }
}
