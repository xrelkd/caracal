use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Priority {
    Lowest = 1,

    Low = 2,
    #[default]
    Normal = 3,

    High = 4,

    Highest = 5,
}

impl From<i32> for Priority {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Lowest,
            2 => Self::Low,
            4 => Self::High,
            5 => Self::Highest,
            _ => Self::Normal,
        }
    }
}
