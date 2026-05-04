use async_trait::async_trait;
use common::AppError;
use domain::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<User, AppError>;
    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn exists(&self, user_id: &str) -> Result<bool, AppError>;
    async fn find_all(&self) -> Result<Vec<User>, AppError>;
}
