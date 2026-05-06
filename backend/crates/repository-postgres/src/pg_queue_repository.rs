use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::{QueueEntry, QueueStatus};
use repository::QueueRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct QueueEntryRow {
    queue_id:           String,
    user_id:            String,
    show_id:            String,
    requested_seat_ids: Vec<String>,
    status:             String,
    position:           i32,
    created_at:         DateTime<Utc>,
    processed_at:       Option<DateTime<Utc>>,
    conflict_seats:     Option<Vec<String>>,
    booking_id:         Option<String>,
    lock_id:            Option<String>,
}

fn parse_queue_status(s: &str) -> QueueStatus {
    match s {
        "Processing" => QueueStatus::Processing,
        "Locked"     => QueueStatus::Locked,
        "Conflict"   => QueueStatus::Conflict,
        "Expired"    => QueueStatus::Expired,
        _            => QueueStatus::Waiting,
    }
}

impl From<QueueEntryRow> for QueueEntry {
    fn from(r: QueueEntryRow) -> Self {
        Self {
            queue_id:           r.queue_id,
            user_id:            r.user_id,
            show_id:            r.show_id,
            requested_seat_ids: r.requested_seat_ids,
            status:             parse_queue_status(&r.status),
            position:           r.position as u32,
            created_at:         r.created_at,
            processed_at:       r.processed_at,
            conflict_seats:     r.conflict_seats,
            booking_id:         r.booking_id,
            lock_id:            r.lock_id,
        }
    }
}

pub struct PgQueueRepository {
    pool: PgPool,
}

impl PgQueueRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl QueueRepository for PgQueueRepository {
    async fn save(&self, entry: QueueEntry) -> Result<QueueEntry, AppError> {
        let status = format!("{:?}", entry.status);
        let row = sqlx::query_as::<_, QueueEntryRow>(
            r#"
            INSERT INTO queue_entries
                (queue_id, user_id, show_id, requested_seat_ids, status, position,
                 created_at, processed_at, conflict_seats, booking_id, lock_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (queue_id) DO UPDATE SET
                status         = EXCLUDED.status,
                processed_at   = EXCLUDED.processed_at,
                conflict_seats = EXCLUDED.conflict_seats,
                booking_id     = EXCLUDED.booking_id,
                lock_id        = EXCLUDED.lock_id
            RETURNING *
            "#,
        )
        .bind(&entry.queue_id)
        .bind(&entry.user_id)
        .bind(&entry.show_id)
        .bind(&entry.requested_seat_ids)
        .bind(&status)
        .bind(entry.position as i32)
        .bind(entry.created_at)
        .bind(entry.processed_at)
        .bind(&entry.conflict_seats)
        .bind(&entry.booking_id)
        .bind(&entry.lock_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, queue_id: &str) -> Result<Option<QueueEntry>, AppError> {
        let row = sqlx::query_as::<_, QueueEntryRow>(
            "SELECT * FROM queue_entries WHERE queue_id = $1",
        )
        .bind(queue_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_waiting_by_show(&self, show_id: &str) -> Result<Vec<QueueEntry>, AppError> {
        let rows = sqlx::query_as::<_, QueueEntryRow>(
            "SELECT * FROM queue_entries WHERE show_id = $1 AND status = 'Waiting' ORDER BY position",
        )
        .bind(show_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: QueueStatus,
    ) -> Result<u32, AppError> {
        let status_str = format!("{:?}", status);
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM queue_entries WHERE show_id = $1 AND status = $2",
        )
        .bind(show_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(count.0 as u32)
    }

    async fn max_position(&self, show_id: &str) -> Result<u32, AppError> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(position) FROM queue_entries WHERE show_id = $1",
        )
        .bind(show_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.0.unwrap_or(0) as u32)
    }

    async fn mark_processed(
        &self,
        queue_id: &str,
        status: QueueStatus,
    ) -> Result<QueueEntry, AppError> {
        let status_str = format!("{:?}", status);
        let row = sqlx::query_as::<_, QueueEntryRow>(
            r#"
            UPDATE queue_entries
            SET status = $2, processed_at = NOW()
            WHERE queue_id = $1
            RETURNING *
            "#,
        )
        .bind(queue_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn delete(&self, queue_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM queue_entries WHERE queue_id = $1")
            .bind(queue_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn find_all_show_ids(&self) -> Result<Vec<String>, AppError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT show_id FROM queue_entries WHERE status IN ('Waiting', 'Processing')",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
