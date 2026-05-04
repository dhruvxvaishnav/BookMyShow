use common::AppConfig;
use repository::{CompensationLogRepository, UserRepository};
use service::booking_service::BookingServiceTrait;
use service::email_service::EmailServiceTrait;
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
    pub compensation_log_repo: Arc<dyn CompensationLogRepository>,
    pub email_svc: Arc<dyn EmailServiceTrait>,
    pub rate_limiter: RateLimiter,
    pub cfg: AppConfig,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seat_locking_svc: Arc<SeatLockingService>,
        booking_svc: Arc<dyn BookingServiceTrait>,
        payment_svc: Arc<dyn PaymentServiceTrait>,
        show_svc: Arc<ShowService>,
        queue_svc: Arc<QueueService>,
        user_repo: Arc<dyn UserRepository>,
        compensation_log_repo: Arc<dyn CompensationLogRepository>,
        email_svc: Arc<dyn EmailServiceTrait>,
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
            compensation_log_repo,
            email_svc,
            rate_limiter,
            cfg,
        }
    }
}
