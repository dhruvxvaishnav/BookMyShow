use async_trait::async_trait;
use common::AppError;
use domain::Payment;
use repository::PaymentRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryPaymentRepository {
    payments: RwLock<HashMap<String, Payment>>,
}

impl InMemoryPaymentRepository {
    pub fn new() -> Self {
        Self { payments: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl PaymentRepository for InMemoryPaymentRepository {
    async fn save(&self, payment: Payment) -> Result<Payment, AppError> {
        let mut w = self.payments.write().await;
        w.insert(payment.payment_id.clone(), payment.clone());
        Ok(payment)
    }

    async fn find_by_id(&self, payment_id: &str) -> Result<Option<Payment>, AppError> {
        let r = self.payments.read().await;
        Ok(r.get(payment_id).cloned())
    }

    async fn find_by_payment_intent_id(
        &self,
        payment_intent_id: &str,
    ) -> Result<Option<Payment>, AppError> {
        let r = self.payments.read().await;
        Ok(r.values()
            .find(|p| p.payment_intent_id == payment_intent_id)
            .cloned())
    }

    async fn find_by_booking(&self, booking_id: &str) -> Result<Option<Payment>, AppError> {
        let r = self.payments.read().await;
        Ok(r.values().find(|p| p.booking_id == booking_id).cloned())
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<Payment>, AppError> {
        let r = self.payments.read().await;
        Ok(r.values().filter(|p| p.user_id == user_id).cloned().collect())
    }
}
