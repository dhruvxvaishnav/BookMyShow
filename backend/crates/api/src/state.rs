use common::AppConfig;
use service::{
    QueueService, SeatLockingService, ShowService,
};
use service::booking_service::BookingServiceTrait;
use service::payment_service::PaymentServiceTrait;
use std::sync::Arc;

/// Aggregates all application services and repositories into a single struct
/// that is passed to every HTTP handler via Axum's extension mechanism.
#[derive(Clone)]
pub struct AppState {
    pub seat_locking_svc: Arc<SeatLockingService>,
    pub booking_svc: Arc<dyn BookingServiceTrait>,
    pub payment_svc: Arc<dyn PaymentServiceTrait>,
    pub show_svc: Arc<ShowService>,
    pub queue_svc: Arc<QueueService>,
    pub cfg: AppConfig,
}

impl AppState {
    pub fn new(
        seat_locking_svc: Arc<SeatLockingService>,
        booking_svc: Arc<dyn BookingServiceTrait>,
        payment_svc: Arc<dyn PaymentServiceTrait>,
        show_svc: Arc<ShowService>,
        queue_svc: Arc<QueueService>,
        cfg: AppConfig,
    ) -> Self {
        Self {
            seat_locking_svc,
            booking_svc,
            payment_svc,
            show_svc,
            queue_svc,
            cfg,
        }
    }
}
