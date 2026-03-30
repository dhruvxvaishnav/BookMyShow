use crate::movie_seat_booking::model::seat::Seat;
use crate::movie_seat_booking::model::show::Show;

pub trait SeatRepository {
    fn save(&mut self, seat: Seat) -> Seat;
    fn find_by_id(&self, seat_id: &str) -> Option<Seat>;
    fn find_by_seat_numbers_and_show(&self, seat_numbers: &[String], show: &Show) -> Vec<Seat>;
    fn get_seats(&self, seat_ids: &[String]) -> Vec<Seat>;
}