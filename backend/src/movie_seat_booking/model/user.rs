#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
}

impl User {
    pub fn new(user_id: String, user_name: String, email: String) -> Self {
        User { user_id, user_name, email }
    }
}