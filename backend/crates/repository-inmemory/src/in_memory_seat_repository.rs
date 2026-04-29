use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::AppError;
use domain::{Seat, SeatStatus, User};
use repository::SeatRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemorySeatRepository {
    seats: RwLock<HashMap<String, Seat>>,
}

impl InMemorySeatRepository {
    pub fn new() -> Self {
        Self {
            seats: RwLock::new(HashMap::new()),
        }
    }

    /// Bulk insert seats (used when generating seat layout for a show).
    pub async fn save_all(&self, seats: Vec<Seat>) -> Result<(), AppError> {
        let mut w = self.seats.write().await;
        for seat in seats {
            w.insert(seat.seat_id.clone(), seat);
        }
        Ok(())
    }
}

#[async_trait]
impl SeatRepository for InMemorySeatRepository {
    async fn save(&self, seat: Seat) -> Result<Seat, AppError> {
        let mut w = self.seats.write().await;
        w.insert(seat.seat_id.clone(), seat.clone());
        Ok(seat)
    }

    async fn find_by_id(&self, seat_id: &str) -> Result<Option<Seat>, AppError> {
        let r = self.seats.read().await;
        Ok(r.get(seat_id).cloned())
    }

    async fn find_by_ids(&self, seat_ids: &[String]) -> Result<Vec<Seat>, AppError> {
        let r = self.seats.read().await;
        Ok(seat_ids
            .iter()
            .filter_map(|id| r.get(id).cloned())
            .collect())
    }

    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Seat>, AppError> {
        let r = self.seats.read().await;
        Ok(r.values()
            .filter(|s| s.show_id == show_id)
            .cloned()
            .collect())
    }

    async fn find_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<Vec<Seat>, AppError> {
        let r = self.seats.read().await;
        Ok(r.values()
            .filter(|s| s.show_id == show_id && s.status == status)
            .cloned()
            .collect())
    }

    async fn lock_seat(
        &self,
        seat_id: &str,
        user_id: &str,
        lock_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Seat, AppError> {
        let mut w = self.seats.write().await;
        let seat = w
            .get_mut(seat_id)
            .ok_or_else(|| AppError::SeatNotFound(seat_id.to_string()))?;

        seat.status = SeatStatus::Locked;
        seat.locked_by = Some(User::new(
            user_id.to_string(),
            user_id.to_string(),
            format!("{user_id}@test.local"),
        ));
        seat.locked_at = Some(Utc::now());
        seat.lock_expires_at = Some(expires_at);
        seat.lock_id = Some(lock_id.to_string());

        Ok(seat.clone())
    }

    async fn release_seat(&self, seat_id: &str) -> Result<Seat, AppError> {
        let mut w = self.seats.write().await;
        let seat = w
            .get_mut(seat_id)
            .ok_or_else(|| AppError::SeatNotFound(seat_id.to_string()))?;

        seat.status = SeatStatus::Available;
        seat.locked_by = None;
        seat.locked_at = None;
        seat.lock_expires_at = None;
        seat.lock_id = None;

        Ok(seat.clone())
    }

    async fn book_seat(&self, seat_id: &str, user_id: &str) -> Result<Seat, AppError> {
        let mut w = self.seats.write().await;
        let seat = w
            .get_mut(seat_id)
            .ok_or_else(|| AppError::SeatNotFound(seat_id.to_string()))?;

        seat.status = SeatStatus::Booked;
        seat.booked_by = Some(User::new(
            user_id.to_string(),
            user_id.to_string(),
            format!("{user_id}@test.local"),
        ));
        // Clear lock fields
        seat.locked_by = None;
        seat.locked_at = None;
        seat.lock_expires_at = None;
        seat.lock_id = None;

        Ok(seat.clone())
    }

    async fn count_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<u32, AppError> {
        let r = self.seats.read().await;
        Ok(r.values()
            .filter(|s| s.show_id == show_id && s.status == status)
            .count() as u32)
    }

    async fn save_all(&self, seats: Vec<Seat>) -> Result<(), AppError> {
        let mut w = self.seats.write().await;
        for seat in seats {
            w.insert(seat.seat_id.clone(), seat);
        }
        Ok(())
    }
}
