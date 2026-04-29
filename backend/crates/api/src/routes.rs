use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let app = Router::new()
        // ── Health ────────────────────────────────────────────────────────────
        .route("/health", get(handlers::health))

        // ── Shows ─────────────────────────────────────────────────────────────
        .route("/shows", get(handlers::list_shows))
        .route("/shows/:show_id", get(handlers::get_show))
        .route("/shows/:show_id/seats", get(handlers::get_seat_layout))
        .route("/shows/:show_id/availability", get(handlers::get_availability))
        .route("/shows/:show_id/seats/lock", post(handlers::lock_seats))
        .route("/shows/:show_id/queue/join", post(handlers::join_queue))

        // ── Bookings ──────────────────────────────────────────────────────────
        .route("/bookings/:booking_id", get(handlers::get_booking))
        .route("/bookings/:booking_id/cancel", post(handlers::cancel_booking))
        .route("/bookings/:booking_id/lock", delete(handlers::release_lock))
        .route("/bookings/:booking_id/extend-lock", post(handlers::extend_lock))
        .route("/bookings/:booking_id/payment/initiate", post(handlers::initiate_payment))
        .route("/bookings/user/:user_id", get(handlers::get_user_bookings))

        // ── Payments ──────────────────────────────────────────────────────────
        .route("/payments/:payment_id", get(handlers::get_payment))
        .route("/payments/callback/:payment_intent_id", post(handlers::payment_callback))

        // ── Queue ─────────────────────────────────────────────────────────────
        .route("/queue/:queue_id", delete(handlers::leave_queue))
        .route("/queue/:queue_id/status", get(handlers::get_queue_status))

        // ── Mock Gateway ──────────────────────────────────────────────────────
        .route("/mock-gateway/pay", post(handlers::mock_gateway_pay))

        // ── Admin ─────────────────────────────────────────────────────────────
        .route("/admin/shows", post(handlers::create_show))

        // Attach app state
        .with_state(state)
        // Add tracing middleware
        .layer(TraceLayer::new_for_http());

    app
}
