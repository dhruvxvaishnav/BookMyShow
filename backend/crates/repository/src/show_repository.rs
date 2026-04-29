use async_trait::async_trait;
use common::AppError;
use domain::Show;

#[async_trait]
pub trait ShowRepository: Send + Sync {
    async fn save(&self, show: Show) -> Result<Show, AppError>;
    async fn find_by_id(&self, show_id: &str) -> Result<Option<Show>, AppError>;
    async fn find_all(&self) -> Result<Vec<Show>, AppError>;
    async fn exists(&self, show_id: &str) -> Result<bool, AppError>;
    async fn delete(&self, show_id: &str) -> Result<(), AppError>;
}
