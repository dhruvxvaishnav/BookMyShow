use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Central application error type. Every business-logic error maps to one of these variants,
/// which in turn maps to a specific HTTP status code.
#[derive(Debug, Error)]
pub enum AppError {
    // ─── 400 Bad Request ────────────────────────────────────────────────────────
    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("seats must belong to the same show")]
    SeatsMustBelongToSameShow,

    #[error("at least one seat must be selected")]
    NoSeatsSelected,

    #[error("maximum {0} seats allowed per booking, got {1}")]
    TooManySeats(usize, usize),

    // ─── 404 Not Found ─────────────────────────────────────────────────────────
    #[error("show not found: {0}")]
    ShowNotFound(String),

    #[error("booking not found: {0}")]
    BookingNotFound(String),

    #[error("seat not found: {0}")]
    SeatNotFound(String),

    #[error("payment not found: {0}")]
    PaymentNotFound(String),

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("lock not found: {0}")]
    LockNotFound(String),

    #[error("queue entry not found: {0}")]
    QueueEntryNotFound(String),

    // ─── 409 Conflict ──────────────────────────────────────────────────────────
    #[error("seats unavailable: {:?}", .0)]
    SeatsUnavailable(Vec<String>),

    #[error("seat unavailable: {0}")]
    SeatUnavailable(String),

    #[error("booking already processed: {0}")]
    BookingAlreadyProcessed(String),

    #[error("lock not owned by user")]
    LockNotOwnedByUser,

    #[error("maximum lock extensions reached")]
    LockMaxExtensionsReached,

    #[error("seats already locked by you")]
    SeatsAlreadyLockedByUser,

    // ─── 410 Gone ───────────────────────────────────────────────────────────────
    #[error("lock expired: {0}")]
    LockExpired(String),

    #[error("booking expired: {0}")]
    BookingExpired(String),

    // ─── 422 Unprocessable Entity ──────────────────────────────────────────────
    #[error("payment amount mismatch: expected {expected}, got {actual}")]
    PaymentMismatch { expected: f64, actual: f64 },

    // ─── 429 Too Many Requests ─────────────────────────────────────────────────
    #[error("rate limit exceeded")]
    RateLimitExceeded,

    // ─── 500 Internal Server Error ─────────────────────────────────────────────
    #[error("internal error: {0}")]
    InternalError(String),

    #[error("repository error: {0}")]
    RepositoryError(String),
}

impl AppError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::ValidationError(_) => 400,
            AppError::SeatsMustBelongToSameShow => 400,
            AppError::NoSeatsSelected => 400,
            AppError::TooManySeats(_, _) => 400,

            AppError::ShowNotFound(_) => 404,
            AppError::BookingNotFound(_) => 404,
            AppError::SeatNotFound(_) => 404,
            AppError::PaymentNotFound(_) => 404,
            AppError::UserNotFound(_) => 404,
            AppError::LockNotFound(_) => 404,
            AppError::QueueEntryNotFound(_) => 404,

            AppError::SeatsUnavailable(_) => 409,
            AppError::SeatUnavailable(_) => 409,
            AppError::BookingAlreadyProcessed(_) => 409,
            AppError::LockNotOwnedByUser => 409,
            AppError::LockMaxExtensionsReached => 409,
            AppError::SeatsAlreadyLockedByUser => 409,

            AppError::LockExpired(_) => 410,
            AppError::BookingExpired(_) => 410,

            AppError::PaymentMismatch { .. } => 422,
            AppError::RateLimitExceeded => 429,

            AppError::InternalError(_) => 500,
            AppError::RepositoryError(_) => 500,
        }
    }

    /// Returns the machine-readable error code string.
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::SeatsMustBelongToSameShow => "SEATS_MUST_BELONG_TO_SAME_SHOW",
            AppError::NoSeatsSelected => "NO_SEATS_SELECTED",
            AppError::TooManySeats(_, _) => "TOO_MANY_SEATS",

            AppError::ShowNotFound(_) => "SHOW_NOT_FOUND",
            AppError::BookingNotFound(_) => "BOOKING_NOT_FOUND",
            AppError::SeatNotFound(_) => "SEAT_NOT_FOUND",
            AppError::PaymentNotFound(_) => "PAYMENT_NOT_FOUND",
            AppError::UserNotFound(_) => "USER_NOT_FOUND",
            AppError::LockNotFound(_) => "LOCK_NOT_FOUND",
            AppError::QueueEntryNotFound(_) => "QUEUE_ENTRY_NOT_FOUND",

            AppError::SeatsUnavailable(_) => "SEATS_UNAVAILABLE",
            AppError::SeatUnavailable(_) => "SEAT_UNAVAILABLE",
            AppError::BookingAlreadyProcessed(_) => "BOOKING_ALREADY_PROCESSED",
            AppError::LockNotOwnedByUser => "LOCK_NOT_OWNED_BY_USER",
            AppError::LockMaxExtensionsReached => "LOCK_MAX_EXTENSIONS_REACHED",
            AppError::SeatsAlreadyLockedByUser => "SEATS_ALREADY_LOCKED_BY_USER",

            AppError::LockExpired(_) => "LOCK_EXPIRED",
            AppError::BookingExpired(_) => "BOOKING_EXPIRED",

            AppError::PaymentMismatch { .. } => "PAYMENT_MISMATCH",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",

            AppError::InternalError(_) => "INTERNAL_ERROR",
            AppError::RepositoryError(_) => "REPOSITORY_ERROR",
        }
    }

    /// Returns additional structured details for specific error variants.
    pub fn error_details(&self) -> Option<HashMap<String, serde_json::Value>> {
        match self {
            AppError::SeatsUnavailable(seat_ids) => {
                let mut m = HashMap::new();
                m.insert("unavailable_seats".to_string(), serde_json::json!(seat_ids));
                Some(m)
            }
            AppError::TooManySeats(max, got) => {
                let mut m = HashMap::new();
                m.insert("max_seats".to_string(), serde_json::json!(max));
                m.insert("requested".to_string(), serde_json::json!(got));
                Some(m)
            }
            AppError::PaymentMismatch { expected, actual } => {
                let mut m = HashMap::new();
                m.insert("expected".to_string(), serde_json::json!(expected));
                m.insert("actual".to_string(), serde_json::json!(actual));
                Some(m)
            }
            _ => None,
        }
    }
}

/// Standard API error response envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Standard API success response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data }
    }
}
