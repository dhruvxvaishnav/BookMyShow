use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{SeatStatus, SeatType, User};

/// Represents a single seat within a show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seat {
    pub seat_id: String,
    /// Human-readable seat number, e.g. "A1", "B12".
    pub seat_number: String,
    /// Row label, e.g. "A", "B".
    pub row_label: String,
    /// Category of seat (Standard / Premium / Recliner).
    #[serde(default)]
    pub seat_type: SeatType,
    /// The show this seat belongs to.
    pub show_id: String,
    /// Current lifecycle status.
    #[serde(default)]
    pub status: SeatStatus,
    /// Set when status == Booked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub booked_by: Option<User>,
    /// User who currently holds the lock. Set when status == Locked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<User>,
    /// When the lock was acquired. Set when status == Locked.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub locked_at: Option<DateTime<Utc>>,
    /// When the lock expires (hard boundary). Set when status == Locked.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub lock_expires_at: Option<DateTime<Utc>>,
    /// Lock session ID. Links this seat to a SeatLock record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_id: Option<String>,
}

mod opt_ts_seconds {
    use chrono::{DateTime, Utc, OutOfRange};
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

impl Seat {
    pub fn new(
        seat_id: String,
        seat_number: String,
        row_label: String,
        seat_type: SeatType,
        show_id: String,
    ) -> Self {
        Self {
            seat_id,
            seat_number,
            row_label,
            seat_type,
            show_id,
            status: SeatStatus::Available,
            booked_by: None,
            locked_by: None,
            locked_at: None,
            lock_expires_at: None,
            lock_id: None,
        }
    }

    /// Calculates the effective price for this seat given the show's base price.
    pub fn effective_price(&self, base_price: f64) -> f64 {
        base_price * self.seat_type.price_modifier()
    }
}
