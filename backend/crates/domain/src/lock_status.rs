use serde::{Deserialize, Serialize};

/// The lifecycle state of a seat lock session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LockStatus {
    /// Lock is active and valid.
    #[default]
    Active,
    /// Lock expired due to TTL elapsed.
    Expired,
    /// Lock was manually released by the user.
    Released,
}

impl std::fmt::Display for LockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockStatus::Active => write!(f, "active"),
            LockStatus::Expired => write!(f, "expired"),
            LockStatus::Released => write!(f, "released"),
        }
    }
}
