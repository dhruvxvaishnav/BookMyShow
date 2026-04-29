use async_trait::async_trait;
use common::AppError;
use domain::{Booking, BookingStatus};

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn save(&self, booking: Booking) -> Result<Booking, AppError>;
    async fn find_by_id(&self, booking_id: &str) -> Result<Option<Booking>, AppError>;
    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Booking>, AppError>;
    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Booking>, AppError>;
    async fn find_by_status(&self, status: BookingStatus) -> Result<Vec<Booking>, AppError>;
    async fn find_by_payment_id(&self, payment_id: &str) -> Result<Option<Booking>, AppError>;
    async fn find_expired(&self, grace_period_secs: i64) -> Result<Vec<Booking>, AppError>;
    async fn find_all(&self) -> Result<Vec<Booking>, AppError>;
}
