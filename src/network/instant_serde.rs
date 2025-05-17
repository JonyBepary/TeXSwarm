use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Convert Instant to DateTime<Utc> for serialization
fn instant_to_datetime(instant: &Instant) -> DateTime<Utc> {
    // This is an approximation as Instant doesn't have a direct conversion to DateTime
    // We use SystemTime::now() as a reference point and calculate the difference
    let now_systemtime = SystemTime::now();
    let now_instant = Instant::now();

    let diff = if *instant <= now_instant {
        now_systemtime
            .checked_sub(Duration::from_nanos(
                (now_instant.duration_since(*instant)).as_nanos() as u64
            ))
            .unwrap_or(UNIX_EPOCH)
    } else {
        now_systemtime
            .checked_add(Duration::from_nanos(
                (instant.duration_since(now_instant)).as_nanos() as u64
            ))
            .unwrap_or(UNIX_EPOCH)
    };

    diff.into()
}

/// Convert DateTime<Utc> back to Instant
fn datetime_to_instant(dt: &DateTime<Utc>) -> Instant {
    let now_systemtime = SystemTime::now();
    let now_instant = Instant::now();
    let dt_systemtime: SystemTime = (*dt).into();

    match dt_systemtime.duration_since(now_systemtime) {
        Ok(future_duration) => now_instant + future_duration,
        Err(past_error) => now_instant - past_error.duration(),
    }
}

/// Serialize an Instant as an ISO 8601 datetime string
pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt = instant_to_datetime(instant);
    dt.serialize(serializer)
}

/// Deserialize an Instant from an ISO 8601 datetime string
pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
where
    D: Deserializer<'de>,
{
    let dt = DateTime::<Utc>::deserialize(deserializer)?;
    Ok(datetime_to_instant(&dt))
}
