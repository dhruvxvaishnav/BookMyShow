use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::AppError;
use domain::{Booking, BookingStatus, Seat};
use repository::BookingRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct BookingRow {
    booking_id: String,
    user_id: String,
    show_id: String,
    seat_ids: Vec<String>,
    status: String,
    payment_id: Option<String>,
    total_amount: f64,
    lock_id: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    seats_snapshot: Option<serde_json::Value>,
}

fn parse_status(s: &str) -> BookingStatus {
    match s {
        "PaymentPending" => BookingStatus::PaymentPending,
        "Success" => BookingStatus::Success,
        "SuccessPartial" => BookingStatus::SuccessPartial,
        "PaymentFailed" => BookingStatus::PaymentFailed,
        "Expired" => BookingStatus::Expired,
        "Cancelled" => BookingStatus::Cancelled,
        "Queued" => BookingStatus::Queued,
        _ => BookingStatus::Pending,
    }
}

impl From<BookingRow> for Booking {
    fn from(r: BookingRow) -> Self {
        let seats_snapshot: Option<Vec<Seat>> = r
            .seats_snapshot
            .and_then(|v| serde_json::from_value(v).ok());
        Self {
            booking_id: r.booking_id,
            user_id: r.user_id,
            show_id: r.show_id,
            seat_ids: r.seat_ids,
            status: parse_status(&r.status),
            payment_id: r.payment_id,
            total_amount: r.total_amount,
            lock_id: r.lock_id,
            created_at: r.created_at,
            expires_at: r.expires_at,
            confirmed_at: r.confirmed_at,
            cancelled_at: r.cancelled_at,
            seats_snapshot,
        }
    }
}

pub struct PgBookingRepository {
    pool: PgPool,
}

impl PgBookingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookingRepository for PgBookingRepository {
    async fn save(&self, booking: Booking) -> Result<Booking, AppError> {
        let status = format!("{:?}", booking.status);
        let snapshot = booking
            .seats_snapshot
            .as_ref()
            .and_then(|s| serde_json::to_value(s).ok());
        let row = sqlx::query_as::<_, BookingRow>(
            r#"
            INSERT INTO bookings
                (booking_id, user_id, show_id, seat_ids, status, payment_id, total_amount,
                 lock_id, created_at, expires_at, confirmed_at, cancelled_at, seats_snapshot)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (booking_id) DO UPDATE SET
                status         = EXCLUDED.status,
                payment_id     = EXCLUDED.payment_id,
                lock_id        = EXCLUDED.lock_id,
                expires_at     = EXCLUDED.expires_at,
                confirmed_at   = EXCLUDED.confirmed_at,
                cancelled_at   = EXCLUDED.cancelled_at,
                seats_snapshot = EXCLUDED.seats_snapshot
            RETURNING *
            "#,
        )
        .bind(&booking.booking_id)
        .bind(&booking.user_id)
        .bind(&booking.show_id)
        .bind(&booking.seat_ids)
        .bind(&status)
        .bind(&booking.payment_id)
        .bind(booking.total_amount)
        .bind(&booking.lock_id)
        .bind(booking.created_at)
        .bind(booking.expires_at)
        .bind(booking.confirmed_at)
        .bind(booking.cancelled_at)
        .bind(snapshot)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, booking_id: &str) -> Result<Option<Booking>, AppError> {
        let row = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings WHERE booking_id = $1")
            .bind(booking_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Booking>, AppError> {
        let rows = sqlx::query_as::<_, BookingRow>(
            "SELECT * FROM bookings WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Booking>, AppError> {
        let rows = sqlx::query_as::<_, BookingRow>(
            "SELECT * FROM bookings WHERE show_id = $1 ORDER BY created_at DESC",
        )
        .bind(show_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_status(&self, status: BookingStatus) -> Result<Vec<Booking>, AppError> {
        let status_str = format!("{:?}", status);
        let rows = sqlx::query_as::<_, BookingRow>(
            "SELECT * FROM bookings WHERE status = $1 ORDER BY created_at DESC",
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_payment_id(&self, payment_id: &str) -> Result<Option<Booking>, AppError> {
        let row = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings WHERE payment_id = $1")
            .bind(payment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_expired(&self, grace_period_secs: i64) -> Result<Vec<Booking>, AppError> {
        let cutoff = Utc::now() - Duration::seconds(grace_period_secs);
        let rows = sqlx::query_as::<_, BookingRow>(
            r#"
            SELECT * FROM bookings
            WHERE expires_at < $1
              AND status IN ('Pending', 'PaymentPending', 'Queued')
            ORDER BY expires_at
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all(&self) -> Result<Vec<Booking>, AppError> {
        let rows =
            sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
