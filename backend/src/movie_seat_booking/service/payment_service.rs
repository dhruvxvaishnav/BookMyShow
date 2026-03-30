pub trait PaymentService {
    fn initiate_payment(&mut self, payment_intent_id: &str, booking_id: &str);
    fn payment_callback(&mut self, payment_intent_id: &str, status: &str);
}