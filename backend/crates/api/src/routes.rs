use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ── Health ────────────────────────────────────────────────────────────
        .route("/health", get(handlers::health))
        // ── Auth ──────────────────────────────────────────────────────────────
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/admin/auth/login", post(handlers::admin_login))
        // ── Shows ─────────────────────────────────────────────────────────────
        .route("/shows", get(handlers::list_shows))
        .route("/shows/:show_id", get(handlers::get_show))
        .route("/shows/:show_id/seats", get(handlers::get_seat_layout))
        .route(
            "/shows/:show_id/availability",
            get(handlers::get_availability),
        )
        .route("/shows/:show_id/seats/lock", post(handlers::lock_seats))
        .route("/shows/:show_id/queue/join", post(handlers::join_queue))
        // ── Bookings ──────────────────────────────────────────────────────────
        .route("/bookings/:booking_id", get(handlers::get_booking))
        .route(
            "/bookings/:booking_id/cancel",
            post(handlers::cancel_booking),
        )
        .route("/bookings/:booking_id/lock", delete(handlers::release_lock))
        .route(
            "/bookings/:booking_id/extend-lock",
            post(handlers::extend_lock),
        )
        .route(
            "/bookings/:booking_id/payment/initiate",
            post(handlers::initiate_payment),
        )
        .route("/bookings/user/:user_id", get(handlers::get_user_bookings))
        // ── Payments ──────────────────────────────────────────────────────────
        .route("/payments/:payment_id", get(handlers::get_payment))
        .route(
            "/payments/:payment_id/refund",
            post(handlers::refund_payment),
        )
        .route(
            "/payments/callback/:payment_intent_id",
            post(handlers::payment_callback),
        )
        .route("/payments/stripe-webhook", post(handlers::stripe_webhook))
        // ── Queue ─────────────────────────────────────────────────────────────
        .route("/queue/:queue_id", delete(handlers::leave_queue))
        .route("/queue/:queue_id/status", get(handlers::get_queue_status))
        // ── Mock Gateway ──────────────────────────────────────────────────────
        .route("/mock-gateway/pay", post(handlers::mock_gateway_pay))
        // ── Admin ─────────────────────────────────────────────────────────────
        .route("/admin/shows", post(handlers::create_show))
        .route("/admin/shows/:show_id", delete(handlers::cancel_show))
        .route(
            "/admin/shows/:show_id/analytics",
            get(handlers::admin_show_analytics),
        )
        .route(
            "/admin/shows/:show_id/seats/:seat_id/override",
            post(handlers::admin_override_seat),
        )
        .route("/admin/bookings", get(handlers::admin_list_bookings))
        // Attach app state
        .with_state(state)
        // Add tracing middleware
        .layer(TraceLayer::new_for_http())
}
