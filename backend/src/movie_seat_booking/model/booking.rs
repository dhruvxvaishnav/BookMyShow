use crate::movie_seat_booking::model::booking_status::BookingStatus;
use crate::movie_seat_booking::model::seat::Seat;
use crate::movie_seat_booking::model::show::Show;
use crate::movie_seat_booking::model::user::User;

#[derive(Debug, Clone)]
pub struct Booking {
    pub booking_id: String,
    pub show: Show,
    pub user: User,
    pub status: BookingStatus,
    pub payment_id: Option<String>,
    pub seats: Vec<Seat>,
}

impl Booking {
    pub fn new(booking_id: String, show: Show, user: User, seats: Vec<Seat>) -> Self {
        Booking {
            booking_id,
            show,
            user,
            status: BookingStatus::Pending,
            payment_id: None,
            seats,
        }
    }
}