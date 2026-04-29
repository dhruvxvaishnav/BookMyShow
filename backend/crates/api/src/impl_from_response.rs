/// Newtype wrapper that implements `IntoResponse` for `AppError`,
/// allowing handlers to return `Result<T, ApiError>` cleanly.
use axum::{Json, http::StatusCode, response::IntoResponse};
use common::error::AppError;
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match &self.0 {
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            AppError::SeatsMustBelongToSameShow => (
                StatusCode::BAD_REQUEST,
                "SEATS_MUST_BELONG_TO_SAME_SHOW",
                self.0.to_string(),
            ),
            AppError::NoSeatsSelected => (
                StatusCode::BAD_REQUEST,
                "NO_SEATS_SELECTED",
                self.0.to_string(),
            ),
            AppError::TooManySeats(_, _) => (
                StatusCode::BAD_REQUEST,
                "TOO_MANY_SEATS",
                self.0.to_string(),
            ),

            AppError::ShowNotFound(_) => {
                (StatusCode::NOT_FOUND, "SHOW_NOT_FOUND", self.0.to_string())
            }
            AppError::BookingNotFound(_) => (
                StatusCode::NOT_FOUND,
                "BOOKING_NOT_FOUND",
                self.0.to_string(),
            ),
            AppError::SeatNotFound(_) => {
                (StatusCode::NOT_FOUND, "SEAT_NOT_FOUND", self.0.to_string())
            }
            AppError::PaymentNotFound(_) => (
                StatusCode::NOT_FOUND,
                "PAYMENT_NOT_FOUND",
                self.0.to_string(),
            ),
            AppError::UserNotFound(_) => {
                (StatusCode::NOT_FOUND, "USER_NOT_FOUND", self.0.to_string())
            }
            AppError::LockNotFound(_) => {
                (StatusCode::NOT_FOUND, "LOCK_NOT_FOUND", self.0.to_string())
            }
            AppError::QueueEntryNotFound(_) => (
                StatusCode::NOT_FOUND,
                "QUEUE_ENTRY_NOT_FOUND",
                self.0.to_string(),
            ),

            AppError::SeatsUnavailable(_) => (
                StatusCode::CONFLICT,
                "SEATS_UNAVAILABLE",
                self.0.to_string(),
            ),
            AppError::SeatUnavailable(_) => {
                (StatusCode::CONFLICT, "SEAT_UNAVAILABLE", self.0.to_string())
            }
            AppError::BookingAlreadyProcessed(_) => (
                StatusCode::CONFLICT,
                "BOOKING_ALREADY_PROCESSED",
                self.0.to_string(),
            ),
            AppError::LockNotOwnedByUser => (
                StatusCode::CONFLICT,
                "LOCK_NOT_OWNED_BY_USER",
                self.0.to_string(),
            ),
            AppError::LockMaxExtensionsReached => (
                StatusCode::CONFLICT,
                "LOCK_MAX_EXTENSIONS_REACHED",
                self.0.to_string(),
            ),
            AppError::SeatsAlreadyLockedByUser => (
                StatusCode::CONFLICT,
                "SEATS_ALREADY_LOCKED_BY_USER",
                self.0.to_string(),
            ),

            AppError::LockExpired(_) => (StatusCode::GONE, "LOCK_EXPIRED", self.0.to_string()),
            AppError::BookingExpired(_) => {
                (StatusCode::GONE, "BOOKING_EXPIRED", self.0.to_string())
            }

            AppError::PaymentMismatch { .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PAYMENT_MISMATCH",
                self.0.to_string(),
            ),

            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                self.0.to_string(),
            ),

            AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                self.0.to_string(),
            ),
            AppError::RepositoryError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "REPOSITORY_ERROR",
                self.0.to_string(),
            ),
        };

        #[derive(Serialize)]
        struct ErrBody {
            success: bool,
            error: ErrDetail,
        }
        #[derive(Serialize)]
        struct ErrDetail {
            code: String,
            message: String,
        }

        let body = ErrBody {
            success: false,
            error: ErrDetail {
                code: code.to_string(),
                message,
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}
