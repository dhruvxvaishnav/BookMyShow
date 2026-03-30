use std::collections::HashMap;
use crate::movie_seat_booking::model::show::Show;
use super::show_repository::ShowRepository;

pub struct InMemoryShowRepository {
    show_map: HashMap<String, Show>,
}

impl InMemoryShowRepository {
    pub fn new() -> Self {
        InMemoryShowRepository { show_map: HashMap::new() }
    }
}

impl ShowRepository for InMemoryShowRepository {
    fn save(&mut self, show: Show) -> Show {
        self.show_map.insert(show.show_id.clone(), show.clone());
        show
    }

    fn find_by_id(&self, show_id: &str) -> Option<Show> {
        self.show_map.get(show_id).cloned()
    }
}