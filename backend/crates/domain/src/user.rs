use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a registered user of the platform.
/// For MVP, users are pre-created (no registration flow).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(user_id: String, user_name: String, email: String) -> Self {
        Self {
            user_id,
            user_name,
            email,
            created_at: Utc::now(),
        }
    }
}
