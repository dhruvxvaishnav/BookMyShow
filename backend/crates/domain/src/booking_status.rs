use serde::{Deserialize, Serialize};

/// The lifecycle state of a booking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    /// Booking created, lock acquired, awaiting payment initiation.
    #[default]
    Pending,
    /// Payment has been initiated, awaiting gateway confirmation.
    PaymentPending,
    /// Payment succeeded and seats are confirmed.
    Success,
    /// Payment failed — seats have been released.
    PaymentFailed,
    /// Booking timed out before payment — seats released.
    Expired,
    /// User cancelled the booking before payment.
    Cancelled,
    /// Booking is currently being processed by the queue.
    Queued,
}

impl std::fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingStatus::Pending => write!(f, "pending"),
            BookingStatus::PaymentPending => write!(f, "payment_pending"),
            BookingStatus::Success => write!(f, "success"),
            BookingStatus::PaymentFailed => write!(f, "payment_failed"),
            BookingStatus::Expired => write!(f, "expired"),
            BookingStatus::Cancelled => write!(f, "cancelled"),
            BookingStatus::Queued => write!(f, "queued"),
        }
    }
}
