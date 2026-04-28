use common::AppConfig;
use domain::{Seat, SeatType, Show};
use repository::{SeatRepository, ShowRepository};
use std::sync::Arc;
use uuid::Uuid;

use super::show::CreateShowRequest;

/// Service for show and seat management (admin-facing).
#[derive(Clone)]
pub struct ShowService {
    show_repo: Arc<dyn ShowRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    cfg: AppConfig,
}

impl ShowService {
    pub fn new(
        show_repo: Arc<dyn ShowRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            show_repo,
            seat_repo,
            cfg,
        }
    }

    /// Create a new show and auto-generate its seat layout.
    pub async fn create_show(
        &self,
        req: CreateShowRequest,
    ) -> Result<(Show, Vec<Seat>), common::AppError> {
        // Validate times
        if req.end_time <= req.start_time {
            return Err(common::AppError::ValidationError(
                "end_time must be after start_time".to_string(),
            ));
        }

        let total_seats: u32 = req.seat_layout.rows.iter().map(|r| r.seats).sum();

        let show = Show::new(
            Uuid::new_v4().to_string(),
            req.show_name.clone(),
            req.theatre_name.clone(),
            req.screen_number,
            req.start_time,
            req.end_time,
            req.price_per_seat,
            total_seats,
        );

        // Generate seats
        let mut seats = Vec::new();
        for row_config in &req.seat_layout.rows {
            let seat_type = match row_config.seat_type.as_str() {
                "premium" => SeatType::Premium,
                "recliner" => SeatType::Recliner,
                _ => SeatType::Standard,
            };

            for seat_num in 1..=row_config.seats {
                let seat_number = format!("{}{}", row_config.row, seat_num);
                let seat = Seat::new(
                    Uuid::new_v4().to_string(),
                    seat_number.clone(),
                    row_config.row.clone(),
                    seat_type,
                    show.show_id.clone(),
                );
                seats.push(seat);
            }
        }

        self.show_repo.save(show.clone()).await?;
        self.seat_repo.save_all(seats.clone()).await?;

        tracing::info!(
            show_id = %show.show_id,
            show_name = %show.show_name,
            total_seats = seats.len(),
            "show created"
        );

        Ok((show, seats))
    }

    /// Get all shows.
    pub async fn list_shows(&self) -> Result<Vec<Show>, common::AppError> {
        self.show_repo.find_all().await
    }

    /// Get a show by ID.
    pub async fn get_show(&self, show_id: &str) -> Result<Option<Show>, common::AppError> {
        self.show_repo.find_by_id(show_id).await
    }

    /// Get all seats for a show with current status.
    pub async fn get_seat_layout(
        &self,
        show_id: &str,
    ) -> Result<Vec<Seat>, common::AppError> {
        // Verify show exists
        self.show_repo
            .find_by_id(show_id)
            .await?
            .ok_or_else(|| common::AppError::ShowNotFound(show_id.to_string()))?;

        self.seat_repo.find_by_show(show_id).await
    }

    /// Get availability summary for a show.
    pub async fn get_show_availability(
        &self,
        show_id: &str,
    ) -> Result<ShowAvailability, common::AppError> {
        let available = self.seat_repo.count_by_show_and_status(show_id, domain::SeatStatus::Available).await?;
        let locked = self.seat_repo.count_by_show_and_status(show_id, domain::SeatStatus::Locked).await?;
        let booked = self.seat_repo.count_by_show_and_status(show_id, domain::SeatStatus::Booked).await?;

        Ok(ShowAvailability {
            show_id: show_id.to_string(),
            available,
            locked,
            booked,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowAvailability {
    pub show_id: String,
    pub available: u32,
    pub locked: u32,
    pub booked: u32,
}
