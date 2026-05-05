use common::AppError;
use domain::Venue;
use repository::VenueRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct VenueService {
    pub venue_repo: Arc<dyn VenueRepository>,
}

impl VenueService {
    pub fn new(venue_repo: Arc<dyn VenueRepository>) -> Self {
        Self { venue_repo }
    }

    pub async fn create_venue(
        &self,
        name: String,
        address: String,
        city: String,
        screen_count: u32,
        amenities: Vec<String>,
    ) -> Result<Venue, AppError> {
        let venue = Venue::new(
            Uuid::new_v4().to_string(),
            name,
            address,
            city,
            screen_count,
            amenities,
        );
        self.venue_repo.save(venue).await
    }

    pub async fn list_venues(&self) -> Result<Vec<Venue>, AppError> {
        let mut venues = self.venue_repo.find_all().await?;
        venues.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(venues)
    }

    pub async fn list_venues_by_city(&self, city: &str) -> Result<Vec<Venue>, AppError> {
        self.venue_repo.find_by_city(city).await
    }

    pub async fn get_venue(&self, venue_id: &str) -> Result<Option<Venue>, AppError> {
        self.venue_repo.find_by_id(venue_id).await
    }
}
