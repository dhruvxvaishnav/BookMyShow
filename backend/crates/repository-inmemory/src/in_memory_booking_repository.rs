use async_trait::async_trait;
use chrono::Utc;
use common::AppError;
use domain::{Booking, BookingStatus};
use repository::BookingRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryBookingRepository {
    bookings: RwLock<HashMap<String, Booking>>,
}

impl InMemoryBookingRepository {
    pub fn new() -> Self {
        Self {
            bookings: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl BookingRepository for InMemoryBookingRepository {
    async fn save(&self, booking: Booking) -> Result<Booking, AppError> {
        let mut w = self.bookings.write().await;
        w.insert(booking.booking_id.clone(), booking.clone());
        Ok(booking)
    }

    async fn find_by_id(&self, booking_id: &str) -> Result<Option<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.get(booking_id).cloned())
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.values()
            .filter(|b| b.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.values()
            .filter(|b| b.show_id == show_id)
            .cloned()
            .collect())
    }

    async fn find_by_status(&self, status: BookingStatus) -> Result<Vec<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.values().filter(|b| b.status == status).cloned().collect())
    }

    async fn find_by_payment_id(&self, payment_id: &str) -> Result<Option<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.values()
            .find(|b| b.payment_id.as_deref() == Some(payment_id))
            .cloned())
    }

    async fn find_all(&self) -> Result<Vec<Booking>, AppError> {
        let r = self.bookings.read().await;
        Ok(r.values().cloned().collect())
    }

    async fn find_expired(&self, grace_period_secs: i64) -> Result<Vec<Booking>, AppError> {
        let r = self.bookings.read().await;
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(grace_period_secs);

        Ok(r.values()
            .filter(|b| {
                b.expires_at < cutoff
                    && matches!(
                        b.status,
                        BookingStatus::Pending
                            | BookingStatus::PaymentPending
                            | BookingStatus::Queued
                    )
            })
            .cloned()
            .collect())
    }
}
