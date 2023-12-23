use std::str::FromStr;

use serde::{
    de,
    de::{Deserialize, Deserializer},
    ser::Serializer,
};

/// # Errors
pub fn serialize<S>(url: &http::Uri, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(url.to_string().as_str())
}

/// # Errors
pub fn deserialize<'de, D>(deserializer: D) -> Result<http::Uri, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    http::Uri::from_str(s.as_str()).map_err(de::Error::custom)
}
