use common::AppConfig;
use repository::UserRepository;
use service::booking_service::BookingServiceTrait;
use service::payment_service::PaymentServiceTrait;
use service::{QueueService, SeatLockingService, ShowService};
use std::sync::Arc;

use super::rate_limiter::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub seat_locking_svc: Arc<SeatLockingService>,
    pub booking_svc: Arc<dyn BookingServiceTrait>,
    pub payment_svc: Arc<dyn PaymentServiceTrait>,
    pub show_svc: Arc<ShowService>,
    pub queue_svc: Arc<QueueService>,
    pub user_repo: Arc<dyn UserRepository>,
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
        user_repo: Arc<dyn UserRepository>,
        rate_limiter: RateLimiter,
        cfg: AppConfig,
    ) -> Self {
        Self {
            seat_locking_svc,
            booking_svc,
            payment_svc,
            show_svc,
            queue_svc,
            user_repo,
            rate_limiter,
            cfg,
        }
    }
}
