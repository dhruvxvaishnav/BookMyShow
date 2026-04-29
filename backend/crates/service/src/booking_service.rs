use async_trait::async_trait;
use chrono::Utc;
use common::AppConfig;
use domain::{Booking, BookingStatus, SeatStatus};
use repository::{BookingRepository, PaymentRepository, SeatRepository};
use std::sync::Arc;

use super::booking::BookingConfirmed;

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
    async fn cancel_booking(
        &self,
        booking_id: &str,
        user_id: &str,
    ) -> Result<(), common::AppError>;

    /// Get a booking by ID.
    async fn get_booking(
        &self,
        booking_id: &str,
    ) -> Result<Option<Booking>, common::AppError>;

    /// List all bookings for a user.
    async fn get_user_bookings(
        &self,
        user_id: &str,
    ) -> Result<Vec<Booking>, common::AppError>;

    async fn get_show_bookings(
        &self,
        show_id: &str,
    ) -> Result<Vec<Booking>, common::AppError>;
}

pub struct BookingService {
    booking_repo: Arc<dyn BookingRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    payment_repo: Arc<dyn PaymentRepository>,
    cfg: AppConfig,
}

impl BookingService {
    pub fn new(
        booking_repo: Arc<dyn BookingRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        payment_repo: Arc<dyn PaymentRepository>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            booking_repo,
            seat_repo,
            payment_repo,
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
            return Err(common::AppError::BookingAlreadyProcessed(booking_id.to_string()));
        }

        // Verify lock hasn't hard-expired (past grace period)
        let now = Utc::now();
        let grace_end = booking.expires_at + chrono::Duration::seconds(self.cfg.seat_lock.grace_period_seconds as i64);
        if now > grace_end {
            return Err(common::AppError::LockExpired(booking.lock_id.clone().unwrap_or_default()));
        }

        // Re-verify all seats are still Locked by this user
        let seats = self.seat_repo.find_by_ids(&booking.seat_ids).await?;
        let mut confirmed_seats: Vec<String> = Vec::new();
        let mut failed_seats: Vec<String> = Vec::new();

        for seat in seats {
            if seat.status == SeatStatus::Locked
                && seat.lock_id.as_ref() == booking.lock_id.as_ref()
            {
                self.seat_repo.book_seat(&seat.seat_id, &booking.user_id).await?;
                confirmed_seats.push(seat.seat_id.clone());
            } else {
                failed_seats.push(seat.seat_id.clone());
            }
        }

        // Update booking status
        let new_status = if failed_seats.is_empty() {
            BookingStatus::Success
        } else {
            BookingStatus::PaymentFailed // partial failure
        };

        booking.status = new_status;
        booking.confirmed_at = Some(Utc::now());
        booking.payment_id = Some(payment_id.to_string());
        self.booking_repo.save(booking.clone()).await?;

        tracing::info!(
            booking_id = %booking_id,
            payment_id = %payment_id,
            confirmed_seats = confirmed_seats.len(),
            failed_seats = failed_seats.len(),
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
            return Err(common::AppError::BookingAlreadyProcessed(booking_id.to_string()));
        }

        // Release all seats
        for seat_id in &booking.seat_ids {
            self.seat_repo.release_seat(seat_id).await?;
        }

        let mut updated = booking;
        updated.status = BookingStatus::Cancelled;
        updated.cancelled_at = Some(Utc::now());
        self.booking_repo.save(updated).await?;

        tracing::info!(booking_id = %booking_id, user_id = %user_id, "booking cancelled");
        Ok(())
    }

    async fn get_booking(&self, booking_id: &str) -> Result<Option<Booking>, common::AppError> {
        self.booking_repo.find_by_id(booking_id).await
    }

    async fn get_user_bookings(
        &self,
        user_id: &str,
    ) -> Result<Vec<Booking>, common::AppError> {
        self.booking_repo.find_by_user(user_id).await
    }

    async fn get_show_bookings(
        &self,
        show_id: &str,
    ) -> Result<Vec<Booking>, common::AppError> {
        self.booking_repo.find_by_show(show_id).await
    }
}
