/// DTO returned after a successful admin seat override (force-release).
#[derive(Debug, Clone)]
pub struct OverrideResult {
    pub seat_id: String,
    pub seat_number: String,
    pub previous_status: String,
    pub new_status: String,
    pub released_lock_id: Option<String>,
}

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
