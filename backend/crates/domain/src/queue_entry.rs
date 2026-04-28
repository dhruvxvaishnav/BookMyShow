use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::QueueStatus;

/// Represents a user's entry in the per-show seat request queue.
/// Used during high-traffic scenarios to order concurrent lock requests fairly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub queue_id: String,
    pub user_id: String,
    pub show_id: String,
    /// Seats the user wants to lock.
    pub requested_seat_ids: Vec<String>,
    pub status: QueueStatus,
    /// Current position in the queue (1-indexed).
    pub position: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Set when the lock is granted or denied.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub processed_at: Option<DateTime<Utc>>,
    /// Set on Conflict — the seats that were unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_seats: Option<Vec<String>>,
    /// Booking ID assigned if lock was successfully acquired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub booking_id: Option<String>,
    /// Lock ID assigned if lock was successfully acquired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_id: Option<String>,
}

mod opt_ts_seconds {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(v: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(dt) => s.serialize_i64(dt.timestamp()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<i64> = Deserialize::deserialize(d)?;
        Ok(opt.map(|ts| DateTime::from_timestamp(ts, 0).unwrap()))
    }
}

impl QueueEntry {
    pub fn new(
        queue_id: String,
        user_id: String,
        show_id: String,
        requested_seat_ids: Vec<String>,
        position: u32,
    ) -> Self {
        Self {
            queue_id,
            user_id,
            show_id,
            requested_seat_ids,
            status: QueueStatus::Waiting,
            position,
            created_at: Utc::now(),
            processed_at: None,
            conflict_seats: None,
            booking_id: None,
            lock_id: None,
        }
    }
}
