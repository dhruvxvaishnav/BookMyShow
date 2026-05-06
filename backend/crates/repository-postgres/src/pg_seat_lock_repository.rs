use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::AppError;
use domain::{LockStatus, SeatLock};
use repository::SeatLockRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct SeatLockRow {
    lock_id:       String,
    user_id:       String,
    show_id:       String,
    seat_ids:      Vec<String>,
    status:        String,
    created_at:    DateTime<Utc>,
    expires_at:    DateTime<Utc>,
    extended_count: i32,
}

impl From<SeatLockRow> for SeatLock {
    fn from(r: SeatLockRow) -> Self {
        let status = match r.status.as_str() {
            "Expired"  => LockStatus::Expired,
            "Released" => LockStatus::Released,
            _          => LockStatus::Active,
        };
        Self {
            lock_id:       r.lock_id,
            user_id:       r.user_id,
            show_id:       r.show_id,
            seat_ids:      r.seat_ids,
            status,
            created_at:    r.created_at,
            expires_at:    r.expires_at,
            extended_count: r.extended_count as u32,
        }
    }
}

pub struct PgSeatLockRepository {
    pool: PgPool,
}

impl PgSeatLockRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl SeatLockRepository for PgSeatLockRepository {
    async fn save(&self, lock: SeatLock) -> Result<SeatLock, AppError> {
        let status = format!("{:?}", lock.status);
        let row = sqlx::query_as::<_, SeatLockRow>(
            r#"
            INSERT INTO seat_locks
                (lock_id, user_id, show_id, seat_ids, status, created_at, expires_at, extended_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (lock_id) DO UPDATE SET
                status         = EXCLUDED.status,
                expires_at     = EXCLUDED.expires_at,
                extended_count = EXCLUDED.extended_count
            RETURNING *
            "#,
        )
        .bind(&lock.lock_id)
        .bind(&lock.user_id)
        .bind(&lock.show_id)
        .bind(&lock.seat_ids)
        .bind(&status)
        .bind(lock.created_at)
        .bind(lock.expires_at)
        .bind(lock.extended_count as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, lock_id: &str) -> Result<Option<SeatLock>, AppError> {
        let row = sqlx::query_as::<_, SeatLockRow>(
            "SELECT * FROM seat_locks WHERE lock_id = $1",
        )
        .bind(lock_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_active_by_show(&self, show_id: &str) -> Result<Vec<SeatLock>, AppError> {
        let rows = sqlx::query_as::<_, SeatLockRow>(
            "SELECT * FROM seat_locks WHERE show_id = $1 AND status = 'Active'",
        )
        .bind(show_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_expired_locks(&self, grace_period_secs: i64) -> Result<Vec<SeatLock>, AppError> {
        let cutoff = Utc::now() - Duration::seconds(grace_period_secs);
        let rows = sqlx::query_as::<_, SeatLockRow>(
            "SELECT * FROM seat_locks WHERE status = 'Active' AND expires_at < $1",
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_status(
        &self,
        lock_id: &str,
        status: LockStatus,
    ) -> Result<SeatLock, AppError> {
        let status_str = format!("{:?}", status);
        let row = sqlx::query_as::<_, SeatLockRow>(
            "UPDATE seat_locks SET status = $2 WHERE lock_id = $1 RETURNING *",
        )
        .bind(lock_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn increment_extension(&self, lock_id: &str) -> Result<SeatLock, AppError> {
        let row = sqlx::query_as::<_, SeatLockRow>(
            "UPDATE seat_locks SET extended_count = extended_count + 1 WHERE lock_id = $1 RETURNING *",
        )
        .bind(lock_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }
}
