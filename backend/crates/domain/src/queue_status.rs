use serde::{Deserialize, Serialize};

/// The lifecycle state of a queue entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueueStatus {
    /// User is waiting in the queue.
    #[default]
    Waiting,
    /// Queue entry is being processed (lock attempt in progress).
    Processing,
    /// Lock was successfully acquired — user proceeds to payment.
    Locked,
    /// One or more requested seats were unavailable — user must re-select.
    Conflict,
    /// Processing window expired without resolution.
    Expired,
}

impl std::fmt::Display for QueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueStatus::Waiting => write!(f, "waiting"),
            QueueStatus::Processing => write!(f, "processing"),
            QueueStatus::Locked => write!(f, "locked"),
            QueueStatus::Conflict => write!(f, "conflict"),
            QueueStatus::Expired => write!(f, "expired"),
        }
    }
}
