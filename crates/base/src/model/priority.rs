use std::{fmt, str::FromStr};

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

impl From<String> for Priority {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "lowest" => Self::Lowest,
            "low" => Self::Low,
            "high" => Self::High,
            "highest" => Self::Highest,
            _ => Self::Normal,
        }
    }
}

impl FromStr for Priority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let priority = match s.to_lowercase().as_str() {
            "lowest" => Self::Lowest,
            "low" => Self::Low,
            "high" => Self::High,
            "highest" => Self::Highest,
            _ => Self::Normal,
        };
        Ok(priority)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Lowest => "lowest",
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Highest => "highest",
        };
        f.write_str(s)
    }
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
