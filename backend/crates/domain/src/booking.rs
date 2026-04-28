use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{BookingStatus, Seat};

/// Represents a user's intent to book one or more seats for a show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub booking_id: String,
    pub user_id: String,
    pub show_id: String,
    /// IDs of all seats included in this booking.
    pub seat_ids: Vec<String>,
    pub status: BookingStatus,
    /// Set when payment has been initiated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,
    /// Total amount to be charged (sum of effective seat prices).
    pub total_amount: f64,
    /// Link to the SeatLock that reserved the seats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_id: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Hard expiry boundary for the booking (derived from lock TTL).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
    /// Set when booking reaches Success state.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Set when booking is cancelled.
    #[serde(skip_serializing_if = "Option::is_none", with = "opt_ts_seconds")]
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Snapshot of seat details at booking time (for confirmation records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seats_snapshot: Option<Vec<Seat>>,
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

impl Booking {
    pub fn new(
        booking_id: String,
        user_id: String,
        show_id: String,
        seat_ids: Vec<String>,
        total_amount: f64,
        lock_id: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            booking_id,
            user_id,
            show_id,
            seat_ids,
            status: BookingStatus::Pending,
            payment_id: None,
            total_amount,
            lock_id: Some(lock_id),
            created_at: Utc::now(),
            expires_at,
            confirmed_at: None,
            cancelled_at: None,
            seats_snapshot: None,
        }
    }

    /// Returns true if this booking is in a state that allows lock manipulation.
    pub fn is_lockable(&self) -> bool {
        matches!(
            self.status,
            BookingStatus::Pending | BookingStatus::PaymentPending | BookingStatus::Queued
        )
    }

    /// Returns true if this booking can proceed to payment.
    pub fn can_pay(&self) -> bool {
        matches!(self.status, BookingStatus::Pending)
    }
}
