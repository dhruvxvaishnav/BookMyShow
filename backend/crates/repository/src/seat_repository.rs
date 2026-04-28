use async_trait::async_trait;
use common::AppError;
use domain::{Seat, SeatStatus};

#[async_trait]
pub trait SeatRepository: Send + Sync {
    /// Persist a seat (insert or update).
    async fn save(&self, seat: Seat) -> Result<Seat, AppError>;

    /// Bulk insert seats (used when generating seat layout for a show).
    async fn save_all(&self, seats: Vec<Seat>) -> Result<(), AppError>;

    /// Find a single seat by ID.
    async fn find_by_id(&self, seat_id: &str) -> Result<Option<Seat>, AppError>;

    /// Find multiple seats by their IDs.
    async fn find_by_ids(&self, seat_ids: &[String]) -> Result<Vec<Seat>, AppError>;

    /// Find all seats for a given show.
    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Seat>, AppError>;

    /// Find all seats for a show filtered by status.
    async fn find_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<Vec<Seat>, AppError>;

    /// Atomically update the status of a seat and set lock/release fields.
    async fn lock_seat(
        &self,
        seat_id: &str,
        user_id: &str,
        lock_id: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Seat, AppError>;

    /// Release a seat back to Available.
    async fn release_seat(&self, seat_id: &str) -> Result<Seat, AppError>;

    /// Mark a seat as Booked.
    async fn book_seat(&self, seat_id: &str, user_id: &str) -> Result<Seat, AppError>;

    /// Count seats by status for a show.
    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<u32, AppError>;
}
