use crate::movie_seat_booking::model::seat_status::SeatStatus;
use crate::movie_seat_booking::model::show::Show;
use crate::movie_seat_booking::model::user::User;

#[derive(Debug, Clone)]
pub struct Seat {
    pub seat_id: String,
    pub seat_number: String,
    pub show: Show,
    pub status: SeatStatus,
    pub booked_by: Option<User>, // None until someone books it
}

impl Seat {
    pub fn new(seat_id: String, seat_number: String, show: Show) -> Self {
        Seat {
            seat_id,
            seat_number,
            show,
            status: SeatStatus::Available, // default
            booked_by: None,
        }
    }
}