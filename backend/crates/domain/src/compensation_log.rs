use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit record created when a booking reaches `SuccessPartial`.
/// Tracks which seats were confirmed vs. which failed, and the amount
/// that should be refunded for the failed portion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationLog {
    pub compensation_id: String,
    pub booking_id: String,
    pub show_id: String,
    pub user_id: String,
    /// Seat IDs that were successfully promoted to Booked.
    pub confirmed_seats: Vec<String>,
    /// Seat IDs that could not be confirmed (stale lock / race).
    pub failed_seats: Vec<String>,
    /// Full booking amount charged.
    pub total_amount: f64,
    /// Pro-rata portion attributable to the failed seats (candidate for refund).
    pub failed_amount: f64,
    pub created_at: DateTime<Utc>,
}

impl CompensationLog {
    pub fn new(
        compensation_id: String,
        booking_id: String,
        show_id: String,
        user_id: String,
        confirmed_seats: Vec<String>,
        failed_seats: Vec<String>,
        total_amount: f64,
    ) -> Self {
        let total_count = confirmed_seats.len() + failed_seats.len();
        let failed_amount = if total_count > 0 {
            total_amount * (failed_seats.len() as f64 / total_count as f64)
        } else {
            0.0
        };

        Self {
            compensation_id,
            booking_id,
            show_id,
            user_id,
            confirmed_seats,
            failed_seats,
            total_amount,
            failed_amount,
            created_at: Utc::now(),
        }
    }
}
