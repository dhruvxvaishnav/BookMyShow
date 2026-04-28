use serde::{Deserialize, Serialize};

/// The lifecycle state of a payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Payment initiated, awaiting gateway response.
    #[default]
    Pending,
    /// Payment confirmed by gateway.
    Success,
    /// Payment declined or errored.
    Failed,
    /// Payment was reversed.
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Success => write!(f, "success"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Refunded => write!(f, "refunded"),
        }
    }
}
