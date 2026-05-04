use async_trait::async_trait;
use common::AppError;
use domain::User;
use repository::UserRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct InMemoryUserRepository {
    users: RwLock<HashMap<String, User>>,
    email_index: RwLock<HashMap<String, String>>, // email → user_id
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            email_index: RwLock::new(HashMap::new()),
        }
    }

    pub async fn seed_test_user(&self) {
        let user = User::new(
            "user-001".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );
        let mut users = self.users.write().await;
        let mut index = self.email_index.write().await;
        index.insert(user.email.clone(), user.user_id.clone());
        users.insert(user.user_id.clone(), user);
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: User) -> Result<User, AppError> {
        let mut users = self.users.write().await;
        let mut index = self.email_index.write().await;
        // Keep email index consistent when user email changes
        if let Some(existing) = users.get(&user.user_id) {
            if existing.email != user.email {
                index.remove(&existing.email);
            }
        }
        index.insert(user.email.clone(), user.user_id.clone());
        users.insert(user.user_id.clone(), user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, AppError> {
        let r = self.users.read().await;
        Ok(r.get(user_id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let index = self.email_index.read().await;
        let users = self.users.read().await;
        Ok(index.get(email).and_then(|id| users.get(id)).cloned())
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
