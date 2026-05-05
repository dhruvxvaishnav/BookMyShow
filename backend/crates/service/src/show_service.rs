use common::AppConfig;
use domain::{Booking, BookingStatus, Seat, SeatType, Show};
use repository::{BookingRepository, SeatRepository, ShowRepository};
use std::sync::Arc;
use uuid::Uuid;

use super::show::CreateShowRequest;

/// Service for show and seat management (admin-facing).
#[derive(Clone)]
pub struct ShowService {
    pub show_repo: Arc<dyn ShowRepository>,
    seat_repo: Arc<dyn SeatRepository>,
    booking_repo: Arc<dyn BookingRepository>,
    #[allow(dead_code)]
    cfg: AppConfig,
}

impl ShowService {
    pub fn new(
        show_repo: Arc<dyn ShowRepository>,
        seat_repo: Arc<dyn SeatRepository>,
        booking_repo: Arc<dyn BookingRepository>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            show_repo,
            seat_repo,
            booking_repo,
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

        let mut show = Show::new(
            Uuid::new_v4().to_string(),
            req.show_name.clone(),
            req.theatre_name.clone(),
            req.screen_number,
            req.start_time,
            req.end_time,
            req.price_per_seat,
            total_seats,
        );
        show.movie_id = req.movie_id.clone();
        show.venue_id = req.venue_id.clone();

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

    /// Cancel a show and delete it. (Bookings will be cancelled by the handler)
    pub async fn cancel_show(&self, show_id: &str) -> Result<(), common::AppError> {
        let _show = self
            .show_repo
            .find_by_id(show_id)
            .await?
            .ok_or_else(|| common::AppError::ShowNotFound(show_id.to_string()))?;

        self.show_repo.delete(show_id).await?;
        Ok(())
    }

    /// Get all seats for a show with current status.
    pub async fn get_seat_layout(&self, show_id: &str) -> Result<Vec<Seat>, common::AppError> {
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
        let available = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Available)
            .await?;
        let locked = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Locked)
            .await?;
        let booked = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Booked)
            .await?;

        Ok(ShowAvailability {
            show_id: show_id.to_string(),
            available,
            locked,
            booked,
        })
    }

    /// Compute occupancy and revenue analytics for a show.
    pub async fn get_show_analytics(
        &self,
        show_id: &str,
    ) -> Result<ShowAnalytics, common::AppError> {
        let show = self
            .show_repo
            .find_by_id(show_id)
            .await?
            .ok_or_else(|| common::AppError::ShowNotFound(show_id.to_string()))?;

        let available = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Available)
            .await?;
        let locked = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Locked)
            .await?;
        let booked = self
            .seat_repo
            .count_by_show_and_status(show_id, domain::SeatStatus::Booked)
            .await?;

        let total = show.total_seats;
        let occupancy_rate = if total > 0 {
            (booked as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Revenue = sum of total_amount for all confirmed (Success) bookings
        let bookings: Vec<Booking> = self.booking_repo.find_by_show(show_id).await?;
        // SuccessPartial counts as revenue (failed portion tracked in CompensationLog)
        let revenue: f64 = bookings
            .iter()
            .filter(|b| {
                matches!(
                    b.status,
                    BookingStatus::Success | BookingStatus::SuccessPartial
                )
            })
            .map(|b| b.total_amount)
            .sum();

        Ok(ShowAnalytics {
            show_id: show_id.to_string(),
            show_name: show.show_name,
            total_seats: total,
            available_seats: available,
            locked_seats: locked,
            booked_seats: booked,
            occupancy_rate,
            revenue,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowAnalytics {
    pub show_id: String,
    pub show_name: String,
    pub total_seats: u32,
    pub available_seats: u32,
    pub locked_seats: u32,
    pub booked_seats: u32,
    pub occupancy_rate: f64,
    pub revenue: f64,
}
