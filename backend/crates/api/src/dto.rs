use serde::{Deserialize, Serialize};

// ─── Auth DTOs ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub user_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user_id: String,
    pub email: String,
    pub user_name: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

/// Request body for locking seats.
#[derive(Debug, Clone, Deserialize)]
pub struct LockSeatsRequest {
    pub seat_ids: Vec<String>,
}

/// Request body for joining the seat selection queue.
#[derive(Debug, Clone, Deserialize)]
pub struct JoinQueueRequest {
    pub seat_ids: Vec<String>,
}

/// Request body for admin show creation.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateShowRequestDto {
    pub show_name: String,
    pub theatre_name: String,
    pub screen_number: u32,
    pub start_time: i64,
    pub end_time: i64,
    pub price_per_seat: f64,
    pub seat_layout: SeatLayoutDto,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeatLayoutDto {
    pub rows: Vec<RowConfigDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RowConfigDto {
    pub row: String,
    pub seats: u32,
    #[serde(default)]
    pub seat_type: String,
}

/// Response envelope for booking lock result.
#[derive(Debug, Clone, Serialize)]
pub struct LockSeatsResponse {
    pub booking_id: String,
    pub lock_id: String,
    pub show_id: String,
    pub seat_ids: Vec<String>,
    pub total_amount: f64,
    pub expires_at: i64,
    pub status: String,
}

/// Response for booking details.
#[derive(Debug, Clone, Serialize)]
pub struct BookingResponse {
    pub booking_id: String,
    pub user_id: String,
    pub show_id: String,
    pub seat_ids: Vec<String>,
    pub status: String,
    pub total_amount: f64,
    pub payment_id: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub confirmed_at: Option<i64>,
    pub cancelled_at: Option<i64>,
}

/// Response for payment initiation.
#[derive(Debug, Clone, Serialize)]
pub struct PaymentInitiatedResponse {
    pub payment_id: String,
    pub payment_intent_id: String,
    pub amount: f64,
    pub gateway_name: String,
    pub status: String,
}

/// Response for payment details.
#[derive(Debug, Clone, Serialize)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub booking_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub gateway_name: String,
    pub created_at: i64,
}

/// Response for show details.
#[derive(Debug, Clone, Serialize)]
pub struct ShowResponse {
    pub show_id: String,
    pub show_name: String,
    pub theatre_name: String,
    pub screen_number: u32,
    pub start_time: i64,
    pub end_time: i64,
    pub price_per_seat: f64,
    pub total_seats: u32,
}

/// Response for seat details.
#[derive(Debug, Clone, Serialize)]
pub struct SeatResponse {
    pub seat_id: String,
    pub seat_number: String,
    pub row_label: String,
    pub seat_type: String,
    pub status: String,
    pub lock_expires_at: Option<i64>,
}

/// Response for queue join.
#[derive(Debug, Clone, Serialize)]
pub struct QueueJoinResponse {
    pub queue_id: String,
    pub show_id: String,
    pub position: u32,
    pub status: String,
}

/// Response for queue status.
#[derive(Debug, Clone, Serialize)]
pub struct QueueStatusResponse {
    pub queue_id: String,
    pub status: String,
    pub position: u32,
    pub booking_id: Option<String>,
    pub lock_id: Option<String>,
    pub conflict_seats: Option<Vec<String>>,
}

/// Response for show availability.
#[derive(Debug, Clone, Serialize)]
pub struct AvailabilityResponse {
    pub show_id: String,
    pub available: u32,
    pub locked: u32,
    pub booked: u32,
}

/// Standard paginated seat layout response.
#[derive(Debug, Clone, Serialize)]
pub struct SeatLayoutPageResponse {
    pub show_id: String,
    pub seats: Vec<SeatResponse>,
    pub page: u32,
    pub limit: u32,
}

/// Health check response.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
}

/// Admin: show analytics response.
#[derive(Debug, Clone, Serialize)]
pub struct ShowAnalyticsResponse {
    pub show_id: String,
    pub show_name: String,
    pub total_seats: u32,
    pub available_seats: u32,
    pub locked_seats: u32,
    pub booked_seats: u32,
    pub occupancy_rate: f64,
    pub revenue: f64,
}

/// Admin: seat override request.
#[derive(Debug, Clone, Deserialize)]
pub struct SeatOverrideRequest {
    #[serde(default)]
    pub reason: String,
}

/// Admin: seat override response.
#[derive(Debug, Clone, Serialize)]
pub struct SeatOverrideResponse {
    pub seat_id: String,
    pub seat_number: String,
    pub previous_status: String,
    pub new_status: String,
    pub released_lock_id: Option<String>,
}
