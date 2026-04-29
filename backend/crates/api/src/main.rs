use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, interval};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use api::rate_limiter::RateLimiter;
use api::routes::create_router;
use api::state::AppState;
use common::AppConfig;
use repository::{
    BookingRepository, CompensationLogRepository, PaymentRepository, QueueRepository,
    SeatLockRepository, SeatRepository, ShowRepository, UserRepository,
};
use repository_inmemory::{
    InMemoryBookingRepository, InMemoryCompensationLogRepository, InMemoryPaymentRepository,
    InMemoryQueueRepository, InMemorySeatLockRepository, InMemorySeatRepository,
    InMemoryShowRepository, InMemoryUserRepository,
};
use service::{
    BookingService, PaymentService, QueueService, SeatLockingService, ShowService,
    booking_service::BookingServiceTrait, payment_service::PaymentServiceTrait,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // ── 1. Load configuration ─────────────────────────────────────────────
    let cfg = AppConfig::load().expect("failed to load config");

    // ── 2. Initialise logging ─────────────────────────────────────────────
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.app.log_level));

    // LOG_FORMAT=text switches to human-readable output (useful in dev).
    // Default is JSON for production / structured log ingestion.
    if std::env::var("LOG_FORMAT").as_deref() == Ok("text") {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    }

    tracing::info!(
        host = %cfg.app.host,
        port = %cfg.app.port,
        lock_ttl_secs = %cfg.seat_lock.ttl_seconds,
        "BookMyShow backend starting"
    );

    // ── 3. Initialise repositories (in-memory for now) ─────────────────────
    let in_memory_user_repo = InMemoryUserRepository::new();
    in_memory_user_repo.seed_test_user().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(in_memory_user_repo);
    let show_repo: Arc<dyn ShowRepository> = Arc::new(InMemoryShowRepository::new());
    let seat_repo: Arc<dyn SeatRepository> = Arc::new(InMemorySeatRepository::new());
    let booking_repo: Arc<dyn BookingRepository> = Arc::new(InMemoryBookingRepository::new());
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(InMemoryPaymentRepository::new());
    let seat_lock_repo: Arc<dyn SeatLockRepository> = Arc::new(InMemorySeatLockRepository::new());
    let queue_repo: Arc<dyn QueueRepository> = Arc::new(InMemoryQueueRepository::new());
    let compensation_log_repo: Arc<dyn CompensationLogRepository> =
        Arc::new(InMemoryCompensationLogRepository::new());

    // ── 4. Initialise services ─────────────────────────────────────────────
    let seat_locking_svc = Arc::new(SeatLockingService::new(
        Arc::clone(&show_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&booking_repo),
        Arc::clone(&seat_lock_repo),
        Arc::clone(&user_repo),
        cfg.clone(),
    ));

    let booking_svc = Arc::new(BookingService::new(
        Arc::clone(&booking_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&payment_repo),
        Arc::clone(&compensation_log_repo),
        cfg.clone(),
    ));

    let payment_svc = Arc::new(PaymentService::new(
        Arc::clone(&payment_repo),
        Arc::clone(&booking_repo),
        Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
        cfg.clone(),
    ));

    let show_svc = Arc::new(ShowService::new(
        Arc::clone(&show_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&booking_repo),
        cfg.clone(),
    ));

    let queue_svc = Arc::new(QueueService::new(
        Arc::clone(&queue_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&seat_lock_repo),
        Arc::clone(&seat_locking_svc),
        cfg.clone(),
    ));

    // ── 5. Build app state and router ──────────────────────────────────────
    let rate_limiter = RateLimiter::new();

    let state = AppState::new(
        Arc::clone(&seat_locking_svc),
        Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
        Arc::clone(&payment_svc) as Arc<dyn PaymentServiceTrait>,
        Arc::clone(&show_svc),
        Arc::clone(&queue_svc),
        rate_limiter,
        cfg.clone(),
    );

    // ── 6. Spawn background tasks ──────────────────────────────────────────
    let lock_svc = state.seat_locking_svc.clone();
    let queue_svc = state.queue_svc.clone();
    let payment_svc = state.payment_svc.clone();
    let app = create_router(state);

    // Lock expiration task (every 10s)
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            if let Err(e) = lock_svc.process_expired_locks().await {
                tracing::error!(error = %e, "lock expiration task error");
            }
        }
    });

    // Payment timeout task (every 30s — per PRD §9.5)
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(e) = payment_svc.process_expired_payments().await {
                tracing::error!(error = %e, "payment timeout task error");
            }
        }
    });

    // Queue processor task (polls every 500ms)
    tokio::spawn(async move {
        let poll_interval = Duration::from_millis(500);
        let mut ticker = interval(poll_interval);
        loop {
            ticker.tick().await;
            if let Ok(shows) = queue_svc.queue_repo.find_all_show_ids().await {
                for show_id in shows {
                    if let Err(e) = queue_svc.process_next(&show_id).await {
                        tracing::error!(show_id = %show_id, error = %e, "queue processor error");
                    }
                }
            }
        }
    });

    tracing::info!("background tasks started");

    // ── 7. Start HTTP server ───────────────────────────────────────────────
    let addr = format!("{}:{}", cfg.app.host, cfg.app.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");

    axum::serve(listener, app).await?;

    tracing::info!(uptime = ?start.elapsed(), "server shutdown");
    Ok(())
}
