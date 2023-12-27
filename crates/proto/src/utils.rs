use prost_types::{Timestamp, TimestampError};
use time::{Duration, OffsetDateTime};

/// # Errors
///
/// Returns `TimestampError::InvalidDateTime` while failed to parse timestamp
// FIXME: use it
#[allow(dead_code)]
#[inline]
pub fn timestamp_to_datetime(timestamp: &Timestamp) -> Result<OffsetDateTime, TimestampError> {
    let datetime = OffsetDateTime::from_unix_timestamp(timestamp.seconds)
        .map_err(|_| TimestampError::InvalidDateTime)?;
    let nanos = Duration::nanoseconds(i64::from(timestamp.nanos));

    Ok(datetime + nanos)
}

// SAFETY: it will never panic because nanos may not exceed 999_999_999 which is
// less than i32::MAX
// FIXME: use it
#[allow(clippy::missing_panics_doc, dead_code)]
#[inline]
#[must_use]
pub fn datetime_to_timestamp(dt: &OffsetDateTime) -> Timestamp {
    let seconds = dt.unix_timestamp();
    let nanos = i32::try_from(dt.nanosecond())
        .expect("nanos may not exceed 999_999_999 which is less than i32::MAX");

    Timestamp { seconds, nanos }
}
