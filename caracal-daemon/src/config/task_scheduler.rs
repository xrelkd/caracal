use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskSchedulerConfig {
    #[serde(default = "TaskSchedulerConfig::default_concurrent_number")]
    pub concurrent_number: usize,
}

impl Default for TaskSchedulerConfig {
    fn default() -> Self { Self { concurrent_number: Self::default_concurrent_number() } }
}

impl TaskSchedulerConfig {
    pub const fn default_concurrent_number() -> usize { 10 }
}
