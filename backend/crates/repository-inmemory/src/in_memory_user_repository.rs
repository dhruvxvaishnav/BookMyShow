use async_trait::async_trait;
use common::AppError;
use domain::User;
use repository::UserRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryUserRepository {
    users: RwLock<HashMap<String, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self { users: RwLock::new(HashMap::new()) }
    }

    /// Pre-seed with a test user for development.
    pub async fn seed_test_user(&self) {
        let user = User::new(
            "user-001".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );
        let mut w = self.users.write().await;
        w.insert(user.user_id.clone(), user);
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: User) -> Result<User, AppError> {
        let mut w = self.users.write().await;
        w.insert(user.user_id.clone(), user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, AppError> {
        let r = self.users.read().await;
        Ok(r.get(user_id).cloned())
    }

    async fn exists(&self, user_id: &str) -> Result<bool, AppError> {
        let r = self.users.read().await;
        Ok(r.contains_key(user_id))
    }

    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let r = self.users.read().await;
        Ok(r.values().cloned().collect())
    }
}
