use async_trait::async_trait;
use common::AppError;
use domain::Payment;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, payment: Payment) -> Result<Payment, AppError>;
    async fn find_by_id(&self, payment_id: &str) -> Result<Option<Payment>, AppError>;
    async fn find_by_payment_intent_id(
        &self,
        payment_intent_id: &str,
    ) -> Result<Option<Payment>, AppError>;
    async fn find_by_booking(&self, booking_id: &str) -> Result<Option<Payment>, AppError>;
    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Payment>, AppError>;
}
