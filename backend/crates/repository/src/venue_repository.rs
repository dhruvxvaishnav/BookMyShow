use async_trait::async_trait;
use common::AppError;
use domain::Venue;

#[async_trait]
pub trait VenueRepository: Send + Sync {
    async fn save(&self, venue: Venue) -> Result<Venue, AppError>;
    async fn find_by_id(&self, venue_id: &str) -> Result<Option<Venue>, AppError>;
    async fn find_all(&self) -> Result<Vec<Venue>, AppError>;
    async fn find_by_city(&self, city: &str) -> Result<Vec<Venue>, AppError>;
    async fn delete(&self, venue_id: &str) -> Result<(), AppError>;
}
