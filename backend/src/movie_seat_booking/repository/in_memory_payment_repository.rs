use std::collections::HashMap;
use crate::movie_seat_booking::model::payment::Payment;
use crate::movie_seat_booking::model::user::User;
use super::payment_repository::PaymentRepository;

pub struct InMemoryPaymentRepository {
    payment_map: HashMap<String, Payment>,
}

impl InMemoryPaymentRepository {
    pub fn new() -> Self {
        InMemoryPaymentRepository { payment_map: HashMap::new() }
    }
}

impl PaymentRepository for InMemoryPaymentRepository {
    fn save(&mut self, payment: Payment) -> Payment {
        self.payment_map.insert(payment.payment_id.clone(), payment.clone());
        payment
    }

    fn find_by_id(&self, payment_id: &str) -> Option<Payment> {
        self.payment_map.get(payment_id).cloned()
    }

    fn find_by_payment_intent_id(&self, payment_intent_id: &str) -> Option<Payment> {
        self.payment_map.values()
            .find(|p| p.payment_intent_id == payment_intent_id)
            .cloned()
    }

    fn find_by_user(&self, user: &User) -> Vec<Payment> {
        self.payment_map.values()
            .filter(|p| p.user_id == user.user_id)
            .cloned()
            .collect()
    }
}