/// DTO returned after successfully acquiring a seat lock.
#[derive(Debug, Clone)]
pub struct LockResult {
    pub lock_id: String,
    pub booking_id: String,
    pub show_id: String,
    pub seat_ids: Vec<String>,
    pub total_amount: f64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}
