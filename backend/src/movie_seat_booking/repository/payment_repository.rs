use crate::movie_seat_booking::model::payment::Payment;
use crate::movie_seat_booking::model::user::User;

pub trait PaymentRepository {
    fn save(&mut self, payment: Payment) -> Payment;
    fn find_by_id(&self, payment_id: &str) -> Option<Payment>;
    fn find_by_payment_intent_id(&self, payment_intent_id: &str) -> Option<Payment>;
    fn find_by_user(&self, user: &User) -> Vec<Payment>;
}