use crate::movie_seat_booking::model::show::Show;

pub trait ShowRepository {
    fn save(&mut self, show: Show) -> Show;
    fn find_by_id(&self, show_id: &str) -> Option<Show>;
}