use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::{User, UserRole};
use repository::UserRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct UserRow {
    user_id:       String,
    user_name:     String,
    email:         String,
    password_hash: Option<String>,
    role:          String,
    created_at:    DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        Self {
            user_id:       r.user_id,
            user_name:     r.user_name,
            email:         r.email,
            password_hash: r.password_hash,
            role: if r.role == "admin" { UserRole::Admin } else { UserRole::User },
            created_at:    r.created_at,
        }
    }
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn save(&self, user: User) -> Result<User, AppError> {
        let role = user.role.to_string();
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (user_id, user_name, email, password_hash, role, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) DO UPDATE SET
                user_name     = EXCLUDED.user_name,
                email         = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                role          = EXCLUDED.role
            RETURNING *
            "#,
        )
        .bind(&user.user_id)
        .bind(&user.user_name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&role)
        .bind(user.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn exists(&self, user_id: &str) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(count.0 > 0)
    }

    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let rows = sqlx::query_as::<_, UserRow>("SELECT * FROM users ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
