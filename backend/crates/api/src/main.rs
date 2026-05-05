use domain::User;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, interval};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// Observability
use metrics_exporter_prometheus::PrometheusBuilder;
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::{self, Sampler};
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;

use api::rate_limiter::RateLimiter;
use api::routes::create_router;
use api::state::AppState;
use common::AppConfig;
use repository::{
    BookingRepository, CompensationLogRepository, MovieRepository, PaymentRepository,
    QueueRepository, SeatLockRepository, SeatRepository, ShowRepository, UserRepository,
    VenueRepository,
};
use repository_inmemory::{
    InMemoryBookingRepository, InMemoryCompensationLogRepository, InMemoryMovieRepository,
    InMemoryPaymentRepository, InMemoryQueueRepository, InMemorySeatLockRepository,
    InMemorySeatRepository, InMemoryShowRepository, InMemoryUserRepository,
    InMemoryVenueRepository,
};
use service::show::{CreateShowRequest, RowConfig, SeatLayoutRequest};
use service::{
    BookingService, MovieService, PaymentService, QueueService, SeatLockingService, ShowService,
    VenueService, booking_service::BookingServiceTrait, payment_service::PaymentServiceTrait,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // ── 1. Load configuration ─────────────────────────────────────────────
    let cfg = AppConfig::load().expect("failed to load config");

    // ── 2. Initialise Observability & Logging ──────────────────────────────
    // Prometheus Metrics
    let prometheus_port = std::env::var("METRICS_PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse()
        .unwrap_or(9000);
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], prometheus_port))
        .install()
        .expect("failed to install Prometheus recorder");

    // OpenTelemetry Tracing
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "bookmyshow-api")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .ok(); // ok() because OTLP collector might not be present in dev

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.app.log_level));

    // LOG_FORMAT=text switches to human-readable output (useful in dev).
    // Default is JSON for production / structured log ingestion.
    if std::env::var("LOG_FORMAT").as_deref() == Ok("text") {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer());
            
        if let Some(tracer) = tracer {
            subscriber.with(OpenTelemetryLayer::new(tracer)).init();
        } else {
            subscriber.init();
        }
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json());
            
        if let Some(tracer) = tracer {
            subscriber.with(OpenTelemetryLayer::new(tracer)).init();
        } else {
            subscriber.init();
        }
    }

    tracing::info!(
        host = %cfg.app.host,
        port = %cfg.app.port,
        metrics_port = prometheus_port,
        lock_ttl_secs = %cfg.seat_lock.ttl_seconds,
        "BookMyShow backend starting"
    );

    // ── 3. Initialise repositories (in-memory for now) ─────────────────────
    let in_memory_user_repo = InMemoryUserRepository::new();
    in_memory_user_repo.seed_test_user().await;

    // Seed admin user (password from env or default)
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@bookmyshow.com".to_string());
    let admin_pw = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Admin@123".to_string());
    let admin_hash = tokio::task::spawn_blocking(move || {
        bcrypt::hash(&admin_pw, 12).expect("admin password hash failed")
    })
    .await?;
    let admin_user = User::new_admin(
        "admin-001".to_string(),
        "Administrator".to_string(),
        admin_email,
        admin_hash,
    );
    in_memory_user_repo
        .save(admin_user)
        .await
        .expect("seed admin user failed");

    let user_repo: Arc<dyn UserRepository> = Arc::new(in_memory_user_repo);
    let show_repo: Arc<dyn ShowRepository> = Arc::new(InMemoryShowRepository::new());
    let movie_repo: Arc<dyn MovieRepository> = Arc::new(InMemoryMovieRepository::new());
    let venue_repo: Arc<dyn VenueRepository> = Arc::new(InMemoryVenueRepository::new());
    let seat_repo: Arc<dyn SeatRepository> = Arc::new(InMemorySeatRepository::new());
    let booking_repo: Arc<dyn BookingRepository> = Arc::new(InMemoryBookingRepository::new());
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(InMemoryPaymentRepository::new());
    let seat_lock_repo: Arc<dyn SeatLockRepository> = Arc::new(InMemorySeatLockRepository::new());
    let queue_repo: Arc<dyn QueueRepository> = Arc::new(InMemoryQueueRepository::new());
    let compensation_log_repo: Arc<dyn CompensationLogRepository> =
        Arc::new(InMemoryCompensationLogRepository::new());

    // ── 4. Initialise services ─────────────────────────────────────────────
    let seat_locking_svc = Arc::new(
        SeatLockingService::new(
            Arc::clone(&show_repo),
            Arc::clone(&seat_repo),
            Arc::clone(&booking_repo),
            Arc::clone(&seat_lock_repo),
            Arc::clone(&user_repo),
            cfg.clone(),
        )
        .with_audit_log_repo(Arc::clone(&compensation_log_repo)),
    );

    let email_svc = Arc::new(service::EmailService::new(cfg.clone()));

    let booking_svc = Arc::new(BookingService::new(
        Arc::clone(&booking_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&payment_repo),
        Arc::clone(&compensation_log_repo),
        Arc::clone(&user_repo),
        Arc::clone(&show_repo),
        Arc::clone(&email_svc) as Arc<dyn service::EmailServiceTrait>,
        cfg.clone(),
    ));

    let payment_svc = Arc::new(
        PaymentService::new(
            Arc::clone(&payment_repo),
            Arc::clone(&booking_repo),
            Arc::clone(&user_repo),
            Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
            Arc::clone(&email_svc) as Arc<dyn service::EmailServiceTrait>,
            cfg.clone(),
        )
        .with_audit_log_repo(Arc::clone(&compensation_log_repo)),
    );

    let show_svc = Arc::new(ShowService::new(
        Arc::clone(&show_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&booking_repo),
        cfg.clone(),
    ));

    let movie_svc = Arc::new(MovieService::new(
        Arc::clone(&movie_repo),
        Arc::clone(&show_repo),
    ));

    let venue_svc = Arc::new(VenueService::new(Arc::clone(&venue_repo)));

    let queue_svc = Arc::new(QueueService::new(
        Arc::clone(&queue_repo),
        Arc::clone(&seat_repo),
        Arc::clone(&seat_lock_repo),
        Arc::clone(&seat_locking_svc),
        cfg.clone(),
    ));

    // ── 4b. Seed demo data ───────────────────────────────────────────────────
    seed_demo_data(&show_svc, &movie_svc, &venue_svc).await;

    // ── 5. Build app state and router ──────────────────────────────────────
    let rate_limiter = RateLimiter::new();

    let state = AppState::new(
        Arc::clone(&seat_locking_svc),
        Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
        Arc::clone(&payment_svc) as Arc<dyn PaymentServiceTrait>,
        Arc::clone(&show_svc),
        Arc::clone(&queue_svc),
        Arc::clone(&movie_svc),
        Arc::clone(&venue_svc),
        Arc::clone(&user_repo),
        Arc::clone(&compensation_log_repo),
        Arc::clone(&email_svc) as Arc<dyn service::EmailServiceTrait>,
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

/// Seed demo movies, venues, and shows so the home page isn't empty on first boot.
async fn seed_demo_data(
    show_svc: &ShowService,
    movie_svc: &MovieService,
    venue_svc: &VenueService,
) {
    let existing = show_svc.list_shows().await.unwrap_or_default();
    if !existing.is_empty() {
        tracing::info!(count = existing.len(), "shows already exist, skipping seed");
        return;
    }

    // ── Seed venues ──────────────────────────────────────────────────────────
    let venue_pvr = venue_svc
        .create_venue(
            "PVR Nexus".to_string(),
            "Nexus Mall, Koramangala, Bengaluru".to_string(),
            "Bengaluru".to_string(),
            6,
            vec![
                "Dolby Atmos".to_string(),
                "4DX".to_string(),
                "Food Court".to_string(),
            ],
        )
        .await
        .expect("seed venue PVR failed");

    let venue_imax = venue_svc
        .create_venue(
            "IMAX Central".to_string(),
            "Forum Mall, Koramangala, Bengaluru".to_string(),
            "Bengaluru".to_string(),
            3,
            vec!["IMAX".to_string(), "Laser Projection".to_string()],
        )
        .await
        .expect("seed venue IMAX failed");

    let venue_gold = venue_svc
        .create_venue(
            "Gold Class Cinemas".to_string(),
            "Phoenix MarketCity, Whitefield, Bengaluru".to_string(),
            "Bengaluru".to_string(),
            2,
            vec![
                "Recliner Seats".to_string(),
                "Premium Dining".to_string(),
                "Valet Parking".to_string(),
            ],
        )
        .await
        .expect("seed venue Gold Class failed");

    let venue_retro = venue_svc
        .create_venue(
            "Retro Cinema".to_string(),
            "Church Street, Bengaluru".to_string(),
            "Bengaluru".to_string(),
            1,
            vec!["Classic Ambience".to_string()],
        )
        .await
        .expect("seed venue Retro failed");

    tracing::info!("seeded 4 demo venues");

    // ── Seed movies ──────────────────────────────────────────────────────────
    let movie_avengers = movie_svc
        .create_movie(
            "Avengers: Endgame".to_string(),
            "Action / Superhero".to_string(),
            "English".to_string(),
            181,
            Some("https://upload.wikimedia.org/wikipedia/en/0/0d/Avengers_Endgame_official_poster.jpg".to_string()),
            8.4,
            "After the devastating events of Infinity War, the universe is in ruins. The Avengers assemble once more in order to reverse Thanos's actions and restore balance to the universe.".to_string(),
        )
        .await
        .expect("seed movie Avengers failed");

    let movie_dune = movie_svc
        .create_movie(
            "Dune: Part Two".to_string(),
            "Sci-Fi / Adventure".to_string(),
            "English".to_string(),
            166,
            Some("https://upload.wikimedia.org/wikipedia/en/5/52/Dune_Part_Two_poster.jpeg".to_string()),
            8.5,
            "Paul Atreides unites with Chani and the Fremen while on a warpath of revenge against the conspirators who destroyed his family.".to_string(),
        )
        .await
        .expect("seed movie Dune failed");

    let movie_spiderman = movie_svc
        .create_movie(
            "Spider-Man: No Way Home".to_string(),
            "Action / Superhero".to_string(),
            "English".to_string(),
            148,
            Some("https://upload.wikimedia.org/wikipedia/en/0/00/Spider-Man_No_Way_Home_poster.jpg".to_string()),
            8.2,
            "With Spider-Man's identity now revealed, Peter asks Doctor Strange for help. When a spell goes wrong, dangerous foes from other worlds start to appear.".to_string(),
        )
        .await
        .expect("seed movie Spider-Man failed");

    let movie_budapest = movie_svc
        .create_movie(
            "The Grand Budapest Hotel".to_string(),
            "Comedy / Drama".to_string(),
            "English".to_string(),
            100,
            Some("https://upload.wikimedia.org/wikipedia/en/1/1c/The_Grand_Budapest_Hotel_Poster.jpg".to_string()),
            8.1,
            "The adventures of Gustave H, a legendary concierge at a famous hotel between the wars, and Zero Moustafa, the lobby boy who becomes his most trusted friend.".to_string(),
        )
        .await
        .expect("seed movie Budapest failed");

    tracing::info!("seeded 4 demo movies");

    // ── Seed shows ───────────────────────────────────────────────────────────
    let now = chrono::Utc::now();
    let hour = chrono::Duration::hours(1);

    let shows = vec![
        CreateShowRequest {
            show_name: "Avengers: Endgame".to_string(),
            theatre_name: "PVR Nexus".to_string(),
            screen_number: 1,
            start_time: now + hour,
            end_time: now + hour * 4,
            price_per_seat: 200.0,
            movie_id: Some(movie_avengers.movie_id.clone()),
            venue_id: Some(venue_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: vec![
                    RowConfig {
                        row: "A".to_string(),
                        seats: 10,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "B".to_string(),
                        seats: 10,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "C".to_string(),
                        seats: 10,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "D".to_string(),
                        seats: 10,
                        seat_type: "Standard".to_string(),
                    },
                ],
            },
        },
        CreateShowRequest {
            show_name: "Dune: Part Two".to_string(),
            theatre_name: "IMAX Central".to_string(),
            screen_number: 2,
            start_time: now + hour * 2,
            end_time: now + hour * 5,
            price_per_seat: 350.0,
            movie_id: Some(movie_dune.movie_id.clone()),
            venue_id: Some(venue_imax.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: vec![
                    RowConfig {
                        row: "A".to_string(),
                        seats: 12,
                        seat_type: "Premium".to_string(),
                    },
                    RowConfig {
                        row: "B".to_string(),
                        seats: 12,
                        seat_type: "Premium".to_string(),
                    },
                    RowConfig {
                        row: "C".to_string(),
                        seats: 12,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "D".to_string(),
                        seats: 12,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "E".to_string(),
                        seats: 12,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "F".to_string(),
                        seats: 12,
                        seat_type: "Recliner".to_string(),
                    },
                ],
            },
        },
        CreateShowRequest {
            show_name: "Spider-Man: No Way Home".to_string(),
            theatre_name: "Gold Class Cinemas".to_string(),
            screen_number: 1,
            start_time: now + hour * 3,
            end_time: now + hour * 6,
            price_per_seat: 500.0,
            movie_id: Some(movie_spiderman.movie_id.clone()),
            venue_id: Some(venue_gold.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: vec![
                    RowConfig {
                        row: "A".to_string(),
                        seats: 8,
                        seat_type: "Recliner".to_string(),
                    },
                    RowConfig {
                        row: "B".to_string(),
                        seats: 8,
                        seat_type: "Recliner".to_string(),
                    },
                    RowConfig {
                        row: "C".to_string(),
                        seats: 8,
                        seat_type: "Recliner".to_string(),
                    },
                ],
            },
        },
        CreateShowRequest {
            show_name: "The Grand Budapest Hotel".to_string(),
            theatre_name: "Retro Cinema".to_string(),
            screen_number: 1,
            start_time: now + hour * 4,
            end_time: now + hour * 6,
            price_per_seat: 150.0,
            movie_id: Some(movie_budapest.movie_id.clone()),
            venue_id: Some(venue_retro.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: vec![
                    RowConfig {
                        row: "A".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "B".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "C".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "D".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "E".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "F".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "G".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                    RowConfig {
                        row: "H".to_string(),
                        seats: 15,
                        seat_type: "Standard".to_string(),
                    },
                ],
            },
        },
    ];

    for show in shows {
        match show_svc.create_show(show).await {
            Ok((s, seats)) => {
                tracing::info!(show_id = %s.show_id, show_name = %s.show_name, seat_count = seats.len(), "seeded demo show");
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to seed demo show");
            }
        }
    }
}
