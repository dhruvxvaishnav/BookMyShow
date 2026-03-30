use std::collections::HashMap;
use crate::movie_seat_booking::model::seat::Seat;
use crate::movie_seat_booking::model::show::Show;
use super::seat_repository::SeatRepository;

pub struct InMemorySeatRepository {
    seat_map: HashMap<String, Seat>,
}

impl InMemorySeatRepository {
    pub fn new() -> Self {
        InMemorySeatRepository { seat_map: HashMap::new() }
    }
}

impl SeatRepository for InMemorySeatRepository {
    fn save(&mut self, seat: Seat) -> Seat {
        self.seat_map.insert(seat.seat_id.clone(), seat.clone());
        seat
    }

    fn find_by_id(&self, seat_id: &str) -> Option<Seat> {
        self.seat_map.get(seat_id).cloned()
    }

    fn find_by_seat_numbers_and_show(&self, seat_numbers: &[String], show: &Show) -> Vec<Seat> {
        self.seat_map.values()
            .filter(|s| s.show.show_id == show.show_id && seat_numbers.contains(&s.seat_number))
            .cloned()
            .collect()
    }

    fn get_seats(&self, seat_ids: &[String]) -> Vec<Seat> {
        self.seat_map.values()
            .filter(|s| seat_ids.contains(&s.seat_id))
            .cloned()
            .collect()
    }
}