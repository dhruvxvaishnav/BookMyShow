use crate::movie_seat_booking::model::booking::Booking;

pub struct SeatBookingResult {
    pub booking_id: String,
    pub payment_intent_id: String,
}

pub trait SeatBookingService {
    fn book_seats(&mut self, show_id: &str, seat_ids: Vec<String>, user_id: &str) -> SeatBookingResult;
    fn confirm_booking(&mut self, booking_id: &str, payment_id: &str) -> Booking;
    fn mark_booking_failed(&mut self, booking_id: &str, payment_id: Option<&str>);
}