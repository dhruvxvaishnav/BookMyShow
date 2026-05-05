use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a movie show / screening in a theatre.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Show {
    pub show_id: String,
    pub show_name: String,
    pub theatre_name: String,
    pub screen_number: u32,
    /// When the show starts (UTC).
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    /// When the show ends (UTC). Must be > start_time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Utc>,
    /// Base price per seat in the show's currency.
    pub price_per_seat: f64,
    /// Total number of seats in this show.
    pub total_seats: u32,
    /// When this show record was created.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Linked movie (optional — enriches the show with film metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movie_id: Option<String>,
    /// Linked venue (optional — enriches the show with theatre location).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue_id: Option<String>,
}

impl Show {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        show_id: String,
        show_name: String,
        theatre_name: String,
        screen_number: u32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        price_per_seat: f64,
        total_seats: u32,
    ) -> Self {
        Self {
            show_id,
            show_name,
            theatre_name,
            screen_number,
            start_time,
            end_time,
            price_per_seat,
            total_seats,
            created_at: Utc::now(),
            movie_id: None,
            venue_id: None,
        }
    }
}
