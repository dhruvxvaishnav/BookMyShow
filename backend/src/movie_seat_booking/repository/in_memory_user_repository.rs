use std::collections::HashMap;
use crate::movie_seat_booking::model::user::User;
use super::user_repository::UserRepository;

pub struct InMemoryUserRepository {
    user_map: HashMap<String, User>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        InMemoryUserRepository { user_map: HashMap::new() }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn save(&mut self, user: User) -> User {
        self.user_map.insert(user.user_id.clone(), user.clone());
        user
    }

    fn find_by_id(&self, user_id: &str) -> Option<User> {
        self.user_map.get(user_id).cloned()
    }
}