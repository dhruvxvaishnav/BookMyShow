use crate::movie_seat_booking::model::user::User;

pub trait UserRepository {
    fn save(&mut self, user: User) -> User;
    fn find_by_id(&self, user_id: &str) -> Option<User>;
}