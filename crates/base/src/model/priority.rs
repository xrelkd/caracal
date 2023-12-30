use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Priority {
    Lowest = 0,

    Low = 1,
    #[default]
    Normal = 2,

    High = 3,

    Highest = 4,
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
            Self::Lowest => "Lowest",
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Highest => "Highest",
        };
        f.write_str(s)
    }
}

impl From<i32> for Priority {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Lowest,
            1 => Self::Low,
            3 => Self::High,
            4 => Self::Highest,
            _ => Self::Normal,
        }
    }
}
