use async_trait::async_trait;
use common::AppError;
use domain::Movie;

#[async_trait]
pub trait MovieRepository: Send + Sync {
    async fn save(&self, movie: Movie) -> Result<Movie, AppError>;
    async fn find_by_id(&self, movie_id: &str) -> Result<Option<Movie>, AppError>;
    async fn find_all(&self) -> Result<Vec<Movie>, AppError>;
    async fn delete(&self, movie_id: &str) -> Result<(), AppError>;
}
