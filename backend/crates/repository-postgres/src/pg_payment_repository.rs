use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::{Payment, PaymentStatus};
use repository::PaymentRepository;
use sqlx::PgPool;

use crate::db_err;

#[derive(sqlx::FromRow)]
struct PaymentRow {
    payment_id:        String,
    payment_intent_id: String,
    booking_id:        String,
    user_id:           String,
    amount:            f64,
    currency:          String,
    status:            String,
    gateway_name:      String,
    gateway_response:  Option<String>,
    idempotency_key:   Option<String>,
    created_at:        DateTime<Utc>,
    confirmed_at:      Option<DateTime<Utc>>,
    failed_at:         Option<DateTime<Utc>>,
    refunded_at:       Option<DateTime<Utc>>,
}

fn parse_payment_status(s: &str) -> PaymentStatus {
    match s {
        "Success"  => PaymentStatus::Success,
        "Failed"   => PaymentStatus::Failed,
        "Refunded" => PaymentStatus::Refunded,
        _          => PaymentStatus::Pending,
    }
}

impl From<PaymentRow> for Payment {
    fn from(r: PaymentRow) -> Self {
        Self {
            payment_id:        r.payment_id,
            payment_intent_id: r.payment_intent_id,
            booking_id:        r.booking_id,
            user_id:           r.user_id,
            amount:            r.amount,
            currency:          r.currency,
            status:            parse_payment_status(&r.status),
            gateway_name:      r.gateway_name,
            gateway_response:  r.gateway_response,
            idempotency_key:   r.idempotency_key,
            created_at:        r.created_at,
            confirmed_at:      r.confirmed_at,
            failed_at:         r.failed_at,
            refunded_at:       r.refunded_at,
        }
    }
}

pub struct PgPaymentRepository {
    pool: PgPool,
}

impl PgPaymentRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl PaymentRepository for PgPaymentRepository {
    async fn save(&self, payment: Payment) -> Result<Payment, AppError> {
        let status = format!("{:?}", payment.status);
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            INSERT INTO payments
                (payment_id, payment_intent_id, booking_id, user_id, amount, currency,
                 status, gateway_name, gateway_response, idempotency_key,
                 created_at, confirmed_at, failed_at, refunded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (payment_id) DO UPDATE SET
                status            = EXCLUDED.status,
                gateway_response  = EXCLUDED.gateway_response,
                confirmed_at      = EXCLUDED.confirmed_at,
                failed_at         = EXCLUDED.failed_at,
                refunded_at       = EXCLUDED.refunded_at
            RETURNING *
            "#,
        )
        .bind(&payment.payment_id)
        .bind(&payment.payment_intent_id)
        .bind(&payment.booking_id)
        .bind(&payment.user_id)
        .bind(payment.amount)
        .bind(&payment.currency)
        .bind(&status)
        .bind(&payment.gateway_name)
        .bind(&payment.gateway_response)
        .bind(&payment.idempotency_key)
        .bind(payment.created_at)
        .bind(payment.confirmed_at)
        .bind(payment.failed_at)
        .bind(payment.refunded_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.into())
    }

    async fn find_by_id(&self, payment_id: &str) -> Result<Option<Payment>, AppError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE payment_id = $1",
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_payment_intent_id(
        &self,
        payment_intent_id: &str,
    ) -> Result<Option<Payment>, AppError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE payment_intent_id = $1",
        )
        .bind(payment_intent_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_booking(&self, booking_id: &str) -> Result<Option<Payment>, AppError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE booking_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(booking_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Payment>, AppError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_idempotency_key(&self, key: &str) -> Result<Option<Payment>, AppError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn find_expired_pending(
        &self,
        before: DateTime<Utc>,
    ) -> Result<Vec<Payment>, AppError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE status = 'Pending' AND created_at < $1",
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
