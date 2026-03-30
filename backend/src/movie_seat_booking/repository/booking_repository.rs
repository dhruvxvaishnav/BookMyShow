use crate::movie_seat_booking::model::booking::Booking;
use crate::movie_seat_booking::model::booking_status::BookingStatus;
use crate::movie_seat_booking::model::show::Show;
use crate::movie_seat_booking::model::user::User;

pub trait BookingRepository {
    fn save(&mut self, booking: Booking) -> Booking;
    fn find_by_id(&self, booking_id: &str) -> Option<Booking>;
    fn find_by_user(&self, user: &User) -> Vec<Booking>;
    fn find_by_show(&self, show: &Show) -> Vec<Booking>;
    fn find_by_status(&self, status: &BookingStatus) -> Vec<Booking>;
    fn find_by_payment_id(&self, payment_id: &str) -> Option<Booking>;
}