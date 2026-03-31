use crate::movie_seat_booking::model::payment_status::PaymentStatus;

#[derive(Debug, Clone)]
pub struct Payment {
    pub payment_id: String,
    pub payment_intent_id: String,
    pub user_id: String,
    pub amount: f64,
    pub booking_id: String,
    pub status: PaymentStatus,
}

impl Payment {
    pub fn new(
        payment_id: String,
        payment_intent_id: String,
        user_id: String,
        amount: f64,
        booking_id: String,
    ) -> Self {
        Payment {
            payment_id,
            payment_intent_id,
            user_id,
            amount,
            booking_id,
            status: PaymentStatus::Pending,
        }
    }
}