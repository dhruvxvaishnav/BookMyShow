use async_trait::async_trait;
use chrono::Utc;
use common::AppConfig;
use domain::{Booking, BookingStatus, CompensationLog, SeatStatus};
use repository::{
    BookingRepository, CompensationLogRepository, PaymentRepository, SeatRepository,
    ShowRepository, UserRepository,
};
use std::sync::Arc;
use uuid::Uuid;

use super::booking::BookingConfirmed;
use crate::email_service::EmailServiceTrait;

/// Trait for the booking service.
#[async_trait]
pub trait BookingServiceTrait: Send + Sync {
    /// Confirm a booking after successful payment.
    /// Verifies the lock is still valid, then promotes seats from Locked → Booked.
    async fn confirm_booking(
        &self,
        booking_id: &str,
        payment_id: &str,
    ) -> Result<BookingConfirmed, common::AppError>;

    /// Cancel a booking (before payment).
    async fn cancel_booking(&self, booking_id: &str, user_id: &str)
    -> Result<(), common::AppError>;

    /// Get a booking by ID.
    async fn get_booking(&self, booking_id: &str) -> Result<Option<Booking>, common::AppError>;

    /// List all bookings for a user.
    async fn get_user_bookings(&self, user_id: &str) -> Result<Vec<Booking>, common::AppError>;

    async fn get_show_bookings(&self, show_id: &str) -> Result<Vec<Booking>, common::AppError>;
    async fn get_all_bookings(&self) -> Result<Vec<Booking>, common::AppError>;
}

pub struct BookingService {
    booking_repo: Arc<dyn BookingRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    payment_repo: Arc<dyn PaymentRepository>,
    compensation_log_repo: Arc<dyn CompensationLogRepository>,
    user_repo: Arc<dyn UserRepository>,
    #[allow(dead_code)]
    show_repo: Arc<dyn ShowRepository>,
    email_svc: Arc<dyn EmailServiceTrait>,
    cfg: AppConfig,
}

impl BookingService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        booking_repo: Arc<dyn BookingRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        payment_repo: Arc<dyn PaymentRepository>,
        compensation_log_repo: Arc<dyn CompensationLogRepository>,
        user_repo: Arc<dyn UserRepository>,
        show_repo: Arc<dyn ShowRepository>,
        email_svc: Arc<dyn EmailServiceTrait>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            booking_repo,
            seat_repo,
            payment_repo,
            compensation_log_repo,
            user_repo,
            show_repo,
            email_svc,
            cfg,
        }
    }
}

#[async_trait]
impl BookingServiceTrait for BookingService {
    async fn confirm_booking(
        &self,
        booking_id: &str,
        payment_id: &str,
    ) -> Result<BookingConfirmed, common::AppError> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| common::AppError::BookingNotFound(booking_id.to_string()))?;

        // Verify payment exists and succeeded
        let payment = self
            .payment_repo
            .find_by_id(payment_id)
            .await?
            .ok_or_else(|| common::AppError::PaymentNotFound(payment_id.to_string()))?;

        // Validate payment amount matches booking
        if (payment.amount - booking.total_amount).abs() > 0.01 {
            return Err(common::AppError::PaymentMismatch {
                expected: booking.total_amount,
                actual: payment.amount,
            });
        }

        // Verify booking is in a confirmable state
        if !matches!(
            booking.status,
            BookingStatus::Pending | BookingStatus::PaymentPending
        ) {
            return Err(common::AppError::BookingAlreadyProcessed(
                booking_id.to_string(),
            ));
        }

        // Verify lock hasn't hard-expired (past grace period)
        let now = Utc::now();
        let grace_end = booking.expires_at
            + chrono::Duration::seconds(self.cfg.seat_lock.grace_period_seconds as i64);
        if now > grace_end {
            return Err(common::AppError::LockExpired(
                booking.lock_id.clone().unwrap_or_default(),
            ));
        }

        // Re-verify all seats are still Locked by this user
        let seats = self.seat_repo.find_by_ids(&booking.seat_ids).await?;
        let mut confirmed_seats: Vec<String> = Vec::new();
        let mut failed_seats: Vec<String> = Vec::new();

        for seat in seats {
            if seat.status == SeatStatus::Locked
                && seat.lock_id.as_ref() == booking.lock_id.as_ref()
            {
                self.seat_repo
                    .book_seat(&seat.seat_id, &booking.user_id)
                    .await?;
                confirmed_seats.push(seat.seat_id.clone());
            } else {
                failed_seats.push(seat.seat_id.clone());
            }
        }

        // Update booking status
        let new_status = if failed_seats.is_empty() {
            BookingStatus::Success
        } else {
            BookingStatus::SuccessPartial
        };

        let previous_status = booking.status.to_string();
        booking.status = new_status;
        booking.confirmed_at = Some(Utc::now());
        booking.payment_id = Some(payment_id.to_string());
        self.booking_repo.save(booking.clone()).await?;

        self.save_audit_event(
            &booking.booking_id,
            &booking.show_id,
            &booking.user_id,
            "book",
            None,
            Some(previous_status),
            Some(new_status.to_string()),
            Some("booking confirmed after successful payment".to_string()),
            Some(serde_json::json!({
                "payment_id": payment_id,
                "confirmed_seats": confirmed_seats.clone(),
                "failed_seats": failed_seats.clone()
            })),
        )
        .await;

        // If partial, record a CompensationLog for the failed seats
        if !failed_seats.is_empty() {
            let comp_log = CompensationLog::new(
                Uuid::new_v4().to_string(),
                booking.booking_id.clone(),
                booking.show_id.clone(),
                booking.user_id.clone(),
                confirmed_seats.clone(),
                failed_seats.clone(),
                booking.total_amount,
            );
            if let Err(e) = self.compensation_log_repo.save(comp_log).await {
                tracing::error!(
                    booking_id = %booking_id,
                    error = %e,
                    "failed to save compensation log"
                );
            }
            tracing::warn!(
                booking_id = %booking_id,
                confirmed = confirmed_seats.len(),
                failed = failed_seats.len(),
                "partial booking — compensation log created"
            );
        }

        tracing::info!(
            booking_id = %booking_id,
            payment_id = %payment_id,
            confirmed_seats = confirmed_seats.len(),
            failed_seats = failed_seats.len(),
            status = %new_status,
            "booking confirmed"
        );

        Ok(BookingConfirmed {
            booking_id: booking.booking_id,
            show_id: booking.show_id,
            seat_ids: confirmed_seats,
            total_amount: booking.total_amount,
            confirmed_at: booking.confirmed_at.unwrap(),
        })
    }

    async fn cancel_booking(
        &self,
        booking_id: &str,
        user_id: &str,
    ) -> Result<(), common::AppError> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| common::AppError::BookingNotFound(booking_id.to_string()))?;

        if booking.user_id != user_id {
            return Err(common::AppError::LockNotOwnedByUser);
        }

        if !booking.is_lockable() {
            return Err(common::AppError::BookingAlreadyProcessed(
                booking_id.to_string(),
            ));
        }

        // Release all seats
        for seat_id in &booking.seat_ids {
            self.seat_repo.release_seat(seat_id).await?;
        }

        let status_from = booking.status.to_string();
        let show_id = booking.show_id.clone();
        let seat_ids = booking.seat_ids.clone();
        let mut updated = booking;
        updated.status = BookingStatus::Cancelled;
        updated.cancelled_at = Some(Utc::now());
        self.booking_repo.save(updated).await?;

        self.save_audit_event(
            booking_id,
            &show_id,
            user_id,
            "cancel",
            Some(user_id.to_string()),
            Some(status_from),
            Some(BookingStatus::Cancelled.to_string()),
            Some("booking cancelled and seats released".to_string()),
            Some(serde_json::json!({ "seat_ids": seat_ids })),
        )
        .await;

        tracing::info!(booking_id = %booking_id, user_id = %user_id, "booking cancelled");

        if let Ok(Some(user)) = self.user_repo.find_by_id(user_id).await {
            let _ = self
                .email_svc
                .send_booking_cancelled(&user.email, booking_id)
                .await;
        }

        Ok(())
    }

    async fn get_booking(&self, booking_id: &str) -> Result<Option<Booking>, common::AppError> {
        self.booking_repo.find_by_id(booking_id).await
    }

    async fn get_user_bookings(&self, user_id: &str) -> Result<Vec<Booking>, common::AppError> {
        self.booking_repo.find_by_user(user_id).await
    }

    async fn get_show_bookings(&self, show_id: &str) -> Result<Vec<Booking>, common::AppError> {
        self.booking_repo.find_by_show(show_id).await
    }

    async fn get_all_bookings(&self) -> Result<Vec<Booking>, common::AppError> {
        self.booking_repo.find_all().await
    }
}

impl BookingService {
    #[allow(clippy::too_many_arguments)]
    async fn save_audit_event(
        &self,
        booking_id: &str,
        show_id: &str,
        user_id: &str,
        event_type: &str,
        actor_id: Option<String>,
        status_from: Option<String>,
        status_to: Option<String>,
        message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let log = CompensationLog::audit_event(
            Uuid::new_v4().to_string(),
            booking_id.to_string(),
            show_id.to_string(),
            user_id.to_string(),
            event_type,
            actor_id,
            status_from,
            status_to,
            message,
            metadata,
        );
        if let Err(e) = self.compensation_log_repo.save(log).await {
            tracing::error!(booking_id = %booking_id, event_type = %event_type, error = %e, "failed to save audit log");
        }
    }
}
