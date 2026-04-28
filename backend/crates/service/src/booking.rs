/// DTO for booking confirmation result.
#[derive(Debug, Clone)]
pub struct BookingConfirmed {
    pub booking_id: String,
    pub show_id: String,
    pub seat_ids: Vec<String>,
    pub total_amount: f64,
    pub confirmed_at: chrono::DateTime<chrono::Utc>,
}
