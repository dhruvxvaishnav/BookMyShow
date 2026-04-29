use common::AppConfig;
use service::booking_service::BookingServiceTrait;
use service::payment_service::PaymentServiceTrait;
use service::{QueueService, SeatLockingService, ShowService};
use std::sync::Arc;

use super::rate_limiter::RateLimiter;

/// Aggregates all application services and repositories into a single struct
/// that is passed to every HTTP handler via Axum's extension mechanism.
#[derive(Clone)]
pub struct AppState {
    pub seat_locking_svc: Arc<SeatLockingService>,
    pub booking_svc: Arc<dyn BookingServiceTrait>,
    pub payment_svc: Arc<dyn PaymentServiceTrait>,
    pub show_svc: Arc<ShowService>,
    pub queue_svc: Arc<QueueService>,
    pub rate_limiter: RateLimiter,
    pub cfg: AppConfig,
}

impl AppState {
    pub fn new(
        seat_locking_svc: Arc<SeatLockingService>,
        booking_svc: Arc<dyn BookingServiceTrait>,
        payment_svc: Arc<dyn PaymentServiceTrait>,
        show_svc: Arc<ShowService>,
        queue_svc: Arc<QueueService>,
        rate_limiter: RateLimiter,
        cfg: AppConfig,
    ) -> Self {
        Self {
            seat_locking_svc,
            booking_svc,
            payment_svc,
            show_svc,
            queue_svc,
            rate_limiter,
            cfg,
        }
    }
}
