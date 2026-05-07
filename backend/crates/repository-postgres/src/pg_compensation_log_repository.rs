use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::CompensationLog;
use repository::CompensationLogRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct CompensationLogRow {
    compensation_id: String,
    booking_id: String,
    show_id: String,
    user_id: String,
    confirmed_seats: Vec<String>,
    failed_seats: Vec<String>,
    total_amount: f64,
    failed_amount: f64,
    event_type: String,
    actor_id: Option<String>,
    status_from: Option<String>,
    status_to: Option<String>,
    message: Option<String>,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

impl From<CompensationLogRow> for CompensationLog {
    fn from(r: CompensationLogRow) -> Self {
        Self {
            compensation_id: r.compensation_id,
            booking_id: r.booking_id,
            show_id: r.show_id,
            user_id: r.user_id,
            confirmed_seats: r.confirmed_seats,
            failed_seats: r.failed_seats,
            total_amount: r.total_amount,
            failed_amount: r.failed_amount,
            event_type: r.event_type,
            actor_id: r.actor_id,
            status_from: r.status_from,
            status_to: r.status_to,
            message: r.message,
            metadata: r.metadata,
            created_at: r.created_at,
        }
    }
}

pub struct PgCompensationLogRepository {
    pool: PgPool,
}

impl PgCompensationLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CompensationLogRepository for PgCompensationLogRepository {
    async fn save(&self, log: CompensationLog) -> Result<CompensationLog, AppError> {
        let row = sqlx::query_as::<_, CompensationLogRow>(
            r#"
            INSERT INTO compensation_logs
                (compensation_id, booking_id, show_id, user_id, confirmed_seats, failed_seats,
                 total_amount, failed_amount, event_type, actor_id, status_from, status_to,
                 message, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (compensation_id) DO UPDATE SET event_type = EXCLUDED.event_type
            RETURNING *
            "#,
        )
        .bind(&log.compensation_id)
        .bind(&log.booking_id)
        .bind(&log.show_id)
        .bind(&log.user_id)
        .bind(&log.confirmed_seats)
        .bind(&log.failed_seats)
        .bind(log.total_amount)
        .bind(log.failed_amount)
        .bind(&log.event_type)
        .bind(&log.actor_id)
        .bind(&log.status_from)
        .bind(&log.status_to)
        .bind(&log.message)
        .bind(&log.metadata)
        .bind(log.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_booking(&self, booking_id: &str) -> Result<Vec<CompensationLog>, AppError> {
        let rows = sqlx::query_as::<_, CompensationLogRow>(
            "SELECT * FROM compensation_logs WHERE booking_id = $1 ORDER BY created_at",
        )
        .bind(booking_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<CompensationLog>, AppError> {
        let rows = sqlx::query_as::<_, CompensationLogRow>(
            "SELECT * FROM compensation_logs WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all(&self) -> Result<Vec<CompensationLog>, AppError> {
        let rows = sqlx::query_as::<_, CompensationLogRow>(
            "SELECT * FROM compensation_logs ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
