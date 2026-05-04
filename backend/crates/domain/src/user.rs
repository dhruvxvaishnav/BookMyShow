use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub role: UserRole,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(user_id: String, user_name: String, email: String) -> Self {
        Self {
            user_id,
            user_name,
            email,
            password_hash: None,
            role: UserRole::User,
            created_at: Utc::now(),
        }
    }

    pub fn new_with_password(
        user_id: String,
        user_name: String,
        email: String,
        password_hash: String,
    ) -> Self {
        Self {
            user_id,
            user_name,
            email,
            password_hash: Some(password_hash),
            role: UserRole::User,
            created_at: Utc::now(),
        }
    }

    pub fn new_admin(
        user_id: String,
        user_name: String,
        email: String,
        password_hash: String,
    ) -> Self {
        Self {
            user_id,
            user_name,
            email,
            password_hash: Some(password_hash),
            role: UserRole::Admin,
            created_at: Utc::now(),
        }
    }
}
