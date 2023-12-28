use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::model::Priority;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskState {
    Pending,
    Downloading,
    Paused,
    Canceled,
    Completed,
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
