use std::{fmt, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::model::Priority;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
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

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTask {
    #[schema(value_type = String, example = "https://httpbin.org/ip")]
    #[serde(with = "crate::serde::uri")]
    pub uri: http::Uri,

    #[schema(value_type = Option<String>, example = "ip.json")]
    pub filename: Option<PathBuf>,

    #[schema(value_type = Option<String>, example = "/tmp")]
    pub output_directory: Option<PathBuf>,

    #[schema(value_type = Option<u64>, example = "5")]
    pub concurrent_number: Option<u64>,

    #[schema(value_type = Option<u64>, example = "null")]
    pub connection_timeout: Option<Duration>,

    pub priority: Priority,

    #[schema(default, value_type = String, example = OffsetDateTime::now_utc)]
    pub creation_timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct TaskStatus {
    #[schema(value_type = u64, example = 20)]
    pub id: u64,

    #[schema(value_type = String, example = "/tmp/data.txt")]
    pub file_path: PathBuf,

    #[schema(value_type = u64, example = 1024)]
    pub content_length: u64,

    pub chunks: Vec<ProgressChunk>,

    #[schema(value_type = u32, example = 5)]
    pub concurrent_number: usize,

    pub state: TaskState,

    pub priority: Priority,

    #[serde(with = "time::serde::rfc3339")]
    #[schema(default, value_type = String, example = OffsetDateTime::now_utc)]
    pub creation_timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
pub struct ProgressChunk {
    #[schema(value_type = u64, example = 50)]
    pub start: u64,

    #[schema(value_type = u64, example = 150)]
    pub end: u64,

    #[schema(value_type = u64, example = 30)]
    pub received: u64,

    #[schema(value_type = bool, example = false)]
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
        len.saturating_sub(self.received)
    }

    #[must_use]
    pub const fn is_completed(&self) -> bool { self.is_completed }
}
