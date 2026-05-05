use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Venue {
    pub venue_id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub screen_count: u32,
    pub amenities: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Venue {
    pub fn new(
        venue_id: String,
        name: String,
        address: String,
        city: String,
        screen_count: u32,
        amenities: Vec<String>,
    ) -> Self {
        Self {
            venue_id,
            name,
            address,
            city,
            screen_count,
            amenities,
            created_at: Utc::now(),
        }
    }
}
