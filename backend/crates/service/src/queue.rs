/// DTO returned when a user joins the queue.
#[derive(Debug, Clone)]
pub struct QueueJoined {
    pub queue_id: String,
    pub show_id: String,
    pub position: u32,
    pub status: String,
}

/// DTO for queue status polling.
#[derive(Debug, Clone)]
pub struct QueueStatusResult {
    pub queue_id: String,
    pub status: String,
    pub position: u32,
    pub booking_id: Option<String>,
    pub lock_id: Option<String>,
    pub conflict_seats: Option<Vec<String>>,
}
