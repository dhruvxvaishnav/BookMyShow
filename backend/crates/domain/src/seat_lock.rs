use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::LockStatus;

/// Represents a seat lock session — a temporary hold on one or more seats
/// while a user proceeds through the payment flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatLock {
    pub lock_id: String,
    pub user_id: String,
    pub show_id: String,
    /// IDs of all seats locked in this session.
    pub seat_ids: Vec<String>,
    pub status: LockStatus,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// When the lock expires (set to created_at + TTL).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
    /// How many times this lock has been extended (capped at MAX_EXTENSIONS).
    pub extended_count: u32,
}

impl SeatLock {
    pub fn new(
        lock_id: String,
        user_id: String,
        show_id: String,
        seat_ids: Vec<String>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            lock_id,
            user_id,
            show_id,
            seat_ids,
            status: LockStatus::Active,
            created_at: Utc::now(),
            expires_at,
            extended_count: 0,
        }
    }

    /// Returns true if the lock is still within its validity window (including grace period).
    pub fn is_active(&self, grace_period_secs: i64) -> bool {
        self.status == LockStatus::Active
            && Utc::now() <= self.expires_at + chrono::Duration::seconds(grace_period_secs)
    }

    /// Returns true if the hard TTL has elapsed (regardless of grace period).
    pub fn is_expired(&self) -> bool {
        self.status == LockStatus::Active && Utc::now() > self.expires_at
    }

    /// Returns true if the lock can be extended further.
    pub fn can_extend(&self, max_extensions: u32) -> bool {
        self.status == LockStatus::Active && self.extended_count < max_extensions
    }
}
