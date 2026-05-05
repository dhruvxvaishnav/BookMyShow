use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    pub movie_id: String,
    pub title: String,
    pub genre: String,
    pub language: String,
    pub duration_minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    pub rating: f32,
    pub description: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Movie {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        movie_id: String,
        title: String,
        genre: String,
        language: String,
        duration_minutes: u32,
        poster_url: Option<String>,
        rating: f32,
        description: String,
    ) -> Self {
        Self {
            movie_id,
            title,
            genre,
            language,
            duration_minutes,
            poster_url,
            rating,
            description,
            created_at: Utc::now(),
        }
    }
}
