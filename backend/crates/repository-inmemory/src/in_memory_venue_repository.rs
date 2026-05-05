use async_trait::async_trait;
use common::AppError;
use domain::Venue;
use repository::VenueRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryVenueRepository {
    venues: RwLock<HashMap<String, Venue>>,
}

impl InMemoryVenueRepository {
    pub fn new() -> Self {
        Self {
            venues: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl VenueRepository for InMemoryVenueRepository {
    async fn save(&self, venue: Venue) -> Result<Venue, AppError> {
        let mut w = self.venues.write().await;
        w.insert(venue.venue_id.clone(), venue.clone());
        Ok(venue)
    }

    async fn find_by_id(&self, venue_id: &str) -> Result<Option<Venue>, AppError> {
        let r = self.venues.read().await;
        Ok(r.get(venue_id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Venue>, AppError> {
        let r = self.venues.read().await;
        Ok(r.values().cloned().collect())
    }

    async fn find_by_city(&self, city: &str) -> Result<Vec<Venue>, AppError> {
        let r = self.venues.read().await;
        let city_lower = city.to_lowercase();
        Ok(r.values()
            .filter(|v| v.city.to_lowercase() == city_lower)
            .cloned()
            .collect())
    }

    async fn delete(&self, venue_id: &str) -> Result<(), AppError> {
        let mut w = self.venues.write().await;
        w.remove(venue_id);
        Ok(())
    }
}
