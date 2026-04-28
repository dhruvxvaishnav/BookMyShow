use serde::{Deserialize, Serialize};

/// The lifecycle state of a seat within a show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SeatStatus {
    /// Seat is free and can be locked or booked.
    #[default]
    Available,
    /// Seat is temporarily held by a user (during payment flow).
    Locked,
    /// Seat is permanently confirmed after successful payment.
    Booked,
}

impl std::fmt::Display for SeatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeatStatus::Available => write!(f, "available"),
            SeatStatus::Locked => write!(f, "locked"),
            SeatStatus::Booked => write!(f, "booked"),
        }
    }
}
