use uuid::{uuid, Uuid};
use crate::movie_seat_booking::model::booking::Booking;
use crate::movie_seat_booking::model::seat_status::SeatStatus;
use crate::movie_seat_booking::repository::show_repository::ShowRepository;
use crate::movie_seat_booking::repository::seat_repository::SeatRepository;
use crate::movie_seat_booking::repository::booking_repository::BookingRepository;
use crate::movie_seat_booking::repository::payment_repository::PaymentRepository;
use crate::movie_seat_booking::repository::user_repository::UserRepository  ;
use super::seat_booking_service::{SeatBookingService, SeatBookingResult };

pub struct SeatBookingServiceImpl{
    show_repository: Box<dyn ShowRepository>,
    booking_repository: Box<dyn BookingRepository>,
    seat_repository: Box<dyn SeatRepository>,
    user_repository: Box<dyn UserRepository>,
}

impl SeatBookingServiceImpl {
    pub fn new(
        show_repository: Box<dyn ShowRepository>,
        booking_repository: Box<dyn BookingRepository>,
        seat_repository: Box<dyn SeatRepository>,
        user_repository: Box<dyn UserRepository>,
    ) -> Self {
        SeatBookingServiceImpl {
            show_repository,
            booking_repository,
            seat_repository,
            user_repository,
        }
    }
}

impl SeatBookingService for SeatBookingServiceImpl {
    fn book_seats(&mut self, show_id: &str, seat_ids: Vec<String>, user_id: &str) -> SeatBookingResult {
        let show = self.show_repository.find_by_id(show_id)
            .expect("Show Does Not Exist");

        let seats = self.seat_repository.get_seats(&seat_ids);
        for seat in &seats {
            if seat.status != SeatStatus::Available {
                panic!("Seat {} is not available", seat.seat_number );
            }
        }

        //Create A Pending Booking
        let booking_id = uuid::Uuid::new_v4().to_string();
        let user = self.user_repository.find_by_id(&user_id)
            .expect("User Not Found");
        let booking = Booking::new(booking_id.clone(), show, user, seats);
        self.booking_repository.save(booking);

        //Create A Payment Intent ID 
        let payment_intent_id = Uuid::new_v4().to_string();
        SeatBookingResult {booking_id, payment_intent_id}
    }
    fn confirm_booking(&mut self, booking_id: &str, payment_id: &str) -> Booking {
        todo!()
    }
    fn mark_booking_failed(&mut self, booking_id: &str, payment_id: Option<&str>) {
        todo!()
    }

}