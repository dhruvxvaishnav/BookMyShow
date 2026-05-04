use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Durable audit/compensation record for booking and payment state transitions.
/// Partial successes also carry seat-level compensation data.
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
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub status_from: Option<String>,
    #[serde(default)]
    pub status_to: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
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
            event_type: "partial_booking".to_string(),
            actor_id: None,
            status_from: None,
            status_to: Some("SuccessPartial".to_string()),
            message: Some("partial booking confirmation requires compensation".to_string()),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn audit_event(
        compensation_id: String,
        booking_id: String,
        show_id: String,
        user_id: String,
        event_type: impl Into<String>,
        actor_id: Option<String>,
        status_from: Option<String>,
        status_to: Option<String>,
        message: Option<String>,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            compensation_id,
            booking_id,
            show_id,
            user_id,
            confirmed_seats: Vec::new(),
            failed_seats: Vec::new(),
            total_amount: 0.0,
            failed_amount: 0.0,
            event_type: event_type.into(),
            actor_id,
            status_from,
            status_to,
            message,
            metadata,
            created_at: Utc::now(),
        }
    }
}
