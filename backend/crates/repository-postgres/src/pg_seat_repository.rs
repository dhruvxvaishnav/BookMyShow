use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::{Seat, SeatStatus, SeatType};
use repository::SeatRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct SeatRow {
    seat_id: String,
    seat_number: String,
    row_label: String,
    seat_type: String,
    show_id: String,
    status: String,
    locked_by_user_id: Option<String>,
    locked_at: Option<DateTime<Utc>>,
    lock_expires_at: Option<DateTime<Utc>>,
    lock_id: Option<String>,
    booked_by_user_id: Option<String>,
}

impl From<SeatRow> for Seat {
    fn from(r: SeatRow) -> Self {
        let status = match r.status.as_str() {
            "Locked" => SeatStatus::Locked,
            "Booked" => SeatStatus::Booked,
            _ => SeatStatus::Available,
        };
        let seat_type = match r.seat_type.as_str() {
            "Comfort" | "Premium" => SeatType::Comfort,
            "Recliner" => SeatType::Recliner,
            _ => SeatType::Standard,
        };
        // locked_by / booked_by embed the full User struct; we only store the ID.
        // Set to None — the service layer doesn't rely on the embedded user object
        // from repository return values.
        let _ = r.locked_by_user_id;
        let _ = r.booked_by_user_id;
        Self {
            seat_id: r.seat_id,
            seat_number: r.seat_number,
            row_label: r.row_label,
            seat_type,
            show_id: r.show_id,
            status,
            booked_by: None,
            locked_by: None,
            locked_at: r.locked_at,
            lock_expires_at: r.lock_expires_at,
            lock_id: r.lock_id,
        }
    }
}

pub struct PgSeatRepository {
    pool: PgPool,
}

impl PgSeatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeatRepository for PgSeatRepository {
    async fn save(&self, seat: Seat) -> Result<Seat, AppError> {
        let status = format!("{:?}", seat.status);
        let seat_type = format!("{:?}", seat.seat_type);
        let locked_by_uid = seat.locked_by.as_ref().map(|u| &u.user_id).cloned();
        let booked_by_uid = seat.booked_by.as_ref().map(|u| &u.user_id).cloned();
        let row = sqlx::query_as::<_, SeatRow>(
            r#"
            INSERT INTO seats
                (seat_id, seat_number, row_label, seat_type, show_id, status,
                 locked_by_user_id, locked_at, lock_expires_at, lock_id, booked_by_user_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (seat_id) DO UPDATE SET
                status             = EXCLUDED.status,
                locked_by_user_id  = EXCLUDED.locked_by_user_id,
                locked_at          = EXCLUDED.locked_at,
                lock_expires_at    = EXCLUDED.lock_expires_at,
                lock_id            = EXCLUDED.lock_id,
                booked_by_user_id  = EXCLUDED.booked_by_user_id
            RETURNING *
            "#,
        )
        .bind(&seat.seat_id)
        .bind(&seat.seat_number)
        .bind(&seat.row_label)
        .bind(&seat_type)
        .bind(&seat.show_id)
        .bind(&status)
        .bind(&locked_by_uid)
        .bind(seat.locked_at)
        .bind(seat.lock_expires_at)
        .bind(&seat.lock_id)
        .bind(&booked_by_uid)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn save_all(&self, seats: Vec<Seat>) -> Result<(), AppError> {
        if seats.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for seat in seats {
            let status = format!("{:?}", seat.status);
            let seat_type = format!("{:?}", seat.seat_type);
            sqlx::query(
                r#"
                INSERT INTO seats
                    (seat_id, seat_number, row_label, seat_type, show_id, status,
                     locked_by_user_id, locked_at, lock_expires_at, lock_id, booked_by_user_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (seat_id) DO NOTHING
                "#,
            )
            .bind(&seat.seat_id)
            .bind(&seat.seat_number)
            .bind(&seat.row_label)
            .bind(&seat_type)
            .bind(&seat.show_id)
            .bind(&status)
            .bind(seat.locked_by.as_ref().map(|u| &u.user_id))
            .bind(seat.locked_at)
            .bind(seat.lock_expires_at)
            .bind(&seat.lock_id)
            .bind(seat.booked_by.as_ref().map(|u| &u.user_id))
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    async fn find_by_id(&self, seat_id: &str) -> Result<Option<Seat>, AppError> {
        let row = sqlx::query_as::<_, SeatRow>("SELECT * FROM seats WHERE seat_id = $1")
            .bind(seat_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_ids(&self, seat_ids: &[String]) -> Result<Vec<Seat>, AppError> {
        if seat_ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query_as::<_, SeatRow>("SELECT * FROM seats WHERE seat_id = ANY($1)")
            .bind(seat_ids)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Seat>, AppError> {
        let rows = sqlx::query_as::<_, SeatRow>(
            "SELECT * FROM seats WHERE show_id = $1 ORDER BY row_label, seat_number",
        )
        .bind(show_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<Vec<Seat>, AppError> {
        let status_str = format!("{:?}", status);
        let rows = sqlx::query_as::<_, SeatRow>(
            "SELECT * FROM seats WHERE show_id = $1 AND status = $2 ORDER BY row_label, seat_number",
        )
        .bind(show_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn lock_seat(
        &self,
        seat_id: &str,
        user_id: &str,
        lock_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Seat, AppError> {
        let row = sqlx::query_as::<_, SeatRow>(
            r#"
            UPDATE seats SET
                status            = 'Locked',
                locked_by_user_id = $2,
                lock_id           = $3,
                locked_at         = NOW(),
                lock_expires_at   = $4
            WHERE seat_id = $1
            RETURNING *
            "#,
        )
        .bind(seat_id)
        .bind(user_id)
        .bind(lock_id)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn release_seat(&self, seat_id: &str) -> Result<Seat, AppError> {
        let row = sqlx::query_as::<_, SeatRow>(
            r#"
            UPDATE seats SET
                status            = 'Available',
                locked_by_user_id = NULL,
                lock_id           = NULL,
                locked_at         = NULL,
                lock_expires_at   = NULL
            WHERE seat_id = $1
            RETURNING *
            "#,
        )
        .bind(seat_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn book_seat(&self, seat_id: &str, user_id: &str) -> Result<Seat, AppError> {
        let row = sqlx::query_as::<_, SeatRow>(
            r#"
            UPDATE seats SET
                status            = 'Booked',
                booked_by_user_id = $2,
                locked_by_user_id = NULL,
                lock_id           = NULL,
                locked_at         = NULL,
                lock_expires_at   = NULL
            WHERE seat_id = $1
            RETURNING *
            "#,
        )
        .bind(seat_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<u32, AppError> {
        let status_str = format!("{:?}", status);
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM seats WHERE show_id = $1 AND status = $2")
                .bind(show_id)
                .bind(&status_str)
                .fetch_one(&self.pool)
                .await
                .map_err(db_err)?;
        Ok(count.0 as u32)
    }
}
