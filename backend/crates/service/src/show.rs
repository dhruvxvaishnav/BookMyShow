/// DTO for creating a new show (admin input).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateShowRequest {
    pub show_name: String,
    pub theatre_name: String,
    pub screen_number: u32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub price_per_seat: f64,
    pub seat_layout: SeatLayoutRequest,
    #[serde(default)]
    pub movie_id: Option<String>,
    #[serde(default)]
    pub venue_id: Option<String>,
}

/// DTO for seat layout in show creation.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SeatLayoutRequest {
    pub rows: Vec<RowConfig>,
}

/// DTO for a single row in the seat layout.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RowConfig {
    pub row: String, // e.g., "A", "B"
    pub seats: u32,  // number of seats in this row
    #[serde(default)]
    pub seat_type: String, // "standard", "premium", "recliner"
}
