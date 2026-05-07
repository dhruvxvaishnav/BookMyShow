use domain::User;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, interval};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// Observability
use metrics_exporter_prometheus::PrometheusBuilder;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{self, Sampler};
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
use repository_postgres::{
    PgBookingRepository, PgCompensationLogRepository, PgMovieRepository, PgPaymentRepository,
    PgQueueRepository, PgSeatLockRepository, PgSeatRepository, PgShowRepository, PgUserRepository,
    PgVenueRepository,
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
    let prometheus_port = std::env::var("METRICS_PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse()
        .unwrap_or(9000);
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], prometheus_port))
        .install()
        .expect("failed to install Prometheus recorder");

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "bookmyshow-api",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .ok();

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.app.log_level));

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
        "BookMyShow backend starting"
    );

    // ── 3. Repositories — postgres if DATABASE_URL is set, otherwise in-memory ──
    let database_url = std::env::var("DATABASE_URL").ok();

    let user_repo: Arc<dyn UserRepository>;
    let show_repo: Arc<dyn ShowRepository>;
    let movie_repo: Arc<dyn MovieRepository>;
    let venue_repo: Arc<dyn VenueRepository>;
    let seat_repo: Arc<dyn SeatRepository>;
    let booking_repo: Arc<dyn BookingRepository>;
    let payment_repo: Arc<dyn PaymentRepository>;
    let seat_lock_repo: Arc<dyn SeatLockRepository>;
    let queue_repo: Arc<dyn QueueRepository>;
    let compensation_log_repo: Arc<dyn CompensationLogRepository>;

    if let Some(db_url) = database_url {
        tracing::info!("connecting to PostgreSQL");
        let pool = sqlx::PgPool::connect(&db_url)
            .await
            .map_err(|e| format!("database connection failed: {e}"))?;

        tracing::info!("running migrations");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| format!("migration failed: {e}"))?;
        tracing::info!("migrations complete");

        user_repo = Arc::new(PgUserRepository::new(pool.clone()));
        show_repo = Arc::new(PgShowRepository::new(pool.clone()));
        movie_repo = Arc::new(PgMovieRepository::new(pool.clone()));
        venue_repo = Arc::new(PgVenueRepository::new(pool.clone()));
        seat_repo = Arc::new(PgSeatRepository::new(pool.clone()));
        booking_repo = Arc::new(PgBookingRepository::new(pool.clone()));
        payment_repo = Arc::new(PgPaymentRepository::new(pool.clone()));
        seat_lock_repo = Arc::new(PgSeatLockRepository::new(pool.clone()));
        queue_repo = Arc::new(PgQueueRepository::new(pool.clone()));
        compensation_log_repo = Arc::new(PgCompensationLogRepository::new(pool.clone()));
    } else {
        tracing::info!("DATABASE_URL not set — using in-memory storage");
        let im_user = InMemoryUserRepository::new();
        im_user.seed_test_user().await;
        user_repo = Arc::new(im_user);
        show_repo = Arc::new(InMemoryShowRepository::new());
        movie_repo = Arc::new(InMemoryMovieRepository::new());
        venue_repo = Arc::new(InMemoryVenueRepository::new());
        seat_repo = Arc::new(InMemorySeatRepository::new());
        booking_repo = Arc::new(InMemoryBookingRepository::new());
        payment_repo = Arc::new(InMemoryPaymentRepository::new());
        seat_lock_repo = Arc::new(InMemorySeatLockRepository::new());
        queue_repo = Arc::new(InMemoryQueueRepository::new());
        compensation_log_repo = Arc::new(InMemoryCompensationLogRepository::new());
    }

    // ── 3b. Seed admin user (idempotent — skipped if already exists) ───────
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@bookmyshow.com".to_string());
    let admin_pw = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Admin@123".to_string());

    if user_repo
        .find_by_id("admin-001")
        .await
        .unwrap_or(None)
        .is_none()
    {
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
        user_repo
            .save(admin_user)
            .await
            .expect("seed admin user failed");
    }

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

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            if let Err(e) = lock_svc.process_expired_locks().await {
                tracing::error!(error = %e, "lock expiration task error");
            }
        }
    });

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(e) = payment_svc.process_expired_payments().await {
                tracing::error!(error = %e, "payment timeout task error");
            }
        }
    });

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

    // ── Venues ────────────────────────────────────────────────────────────────
    macro_rules! venue {
        ($name:expr, $addr:expr, $city:expr, $screens:expr, [$($am:expr),*]) => {
            venue_svc.create_venue(
                $name.to_string(), $addr.to_string(), $city.to_string(), $screens,
                vec![$($am.to_string()),*],
            ).await.expect(concat!("seed venue ", $name, " failed"))
        };
    }

    let v_pvr = venue!(
        "PVR Nexus",
        "Nexus Mall, Koramangala, Bengaluru",
        "Bengaluru",
        6,
        ["Dolby Atmos", "4DX", "Food Court"]
    );
    let v_imax = venue!(
        "IMAX Central",
        "Forum Mall, Koramangala, Bengaluru",
        "Bengaluru",
        3,
        ["IMAX", "Laser Projection"]
    );
    let v_gold = venue!(
        "Gold Class Cinemas",
        "Phoenix MarketCity, Whitefield, Bengaluru",
        "Bengaluru",
        2,
        ["Recliner Seats", "Premium Dining", "Valet Parking"]
    );
    let v_retro = venue!(
        "Retro Cinema",
        "Church Street, Bengaluru",
        "Bengaluru",
        1,
        ["Classic Ambience"]
    );
    let v_pvr_m = venue!(
        "PVR Mumbai",
        "Infinity Mall, Andheri, Mumbai",
        "Mumbai",
        5,
        ["Dolby Atmos", "IMAX", "Café"]
    );
    let v_cinep = venue!(
        "Cinepolis Mumbai",
        "Viviana Mall, Thane, Mumbai",
        "Mumbai",
        6,
        ["4DX", "3D", "VIP Lounge"]
    );
    let v_pvr_d = venue!(
        "PVR Delhi",
        "Select Citywalk, Saket, Delhi",
        "Delhi",
        4,
        ["Dolby Atmos", "Recliner Seats"]
    );
    let v_wave = venue!(
        "Wave Cinemas",
        "Wave City Centre, Noida, Delhi",
        "Delhi",
        3,
        ["3D", "Food Court"]
    );
    let v_inox = venue!(
        "INOX Hyderabad",
        "GVK One Mall, Banjara Hills, Hyderabad",
        "Hyderabad",
        4,
        ["Dolby Atmos", "Premium Lounge"]
    );

    tracing::info!("seeded 9 venues");

    // ── Movies ────────────────────────────────────────────────────────────────
    macro_rules! movie {
        ($title:expr, $genre:expr, $lang:expr, $dur:expr, $poster:expr, $rating:expr, $desc:expr) => {
            movie_svc
                .create_movie(
                    $title.to_string(),
                    $genre.to_string(),
                    $lang.to_string(),
                    $dur,
                    Some($poster.to_string()),
                    $rating,
                    $desc.to_string(),
                )
                .await
                .expect(concat!("seed movie ", $title, " failed"))
        };
    }

    let m_avengers = movie!(
        "Avengers: Endgame",
        "Action / Superhero",
        "English",
        181,
        "https://image.tmdb.org/t/p/w500/or06FN3Dka5tukK1e9sl16pB3iy.jpg",
        8.4,
        "After Thanos wiped out half the universe, the Avengers assemble for one final stand."
    );
    let m_dune = movie!(
        "Dune: Part Two",
        "Sci-Fi / Adventure",
        "English",
        166,
        "https://image.tmdb.org/t/p/w500/1pdfLvkbY9ohJlCjQH2CZjjYVvJ.jpg",
        8.5,
        "Paul Atreides unites with the Fremen on a warpath of revenge against his family's enemies."
    );
    let m_spider = movie!(
        "Spider-Man: No Way Home",
        "Action / Superhero",
        "English",
        148,
        "https://image.tmdb.org/t/p/w500/1g0dhYtq4irTY1GPXvft6k4YLjm.jpg",
        8.2,
        "With his identity revealed, Peter Parker opens the multiverse unleashing dangerous villains."
    );
    let m_budapest = movie!(
        "The Grand Budapest Hotel",
        "Comedy / Drama",
        "English",
        100,
        "https://image.tmdb.org/t/p/w500/eWdyYQreja6JGCzqHWXpWHDrrPo.jpg",
        8.1,
        "A legendary concierge and his protégé navigate intrigue and mystery between the wars."
    );
    let m_oppenheimer = movie!(
        "Oppenheimer",
        "Biography / Drama",
        "English",
        180,
        "https://image.tmdb.org/t/p/w500/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg",
        8.5,
        "The story of J. Robert Oppenheimer and his role in the development of the atomic bomb."
    );
    let m_barbie = movie!(
        "Barbie",
        "Comedy / Fantasy",
        "English",
        114,
        "https://image.tmdb.org/t/p/w500/iuFNMS8U5cb6xfzi51Dbkovj7vM.jpg",
        6.9,
        "Barbie and Ken leave Barbieland on a journey of self-discovery in the real world."
    );
    let m_kgf = movie!(
        "KGF: Chapter 2",
        "Action / Drama",
        "Kannada",
        168,
        "https://image.tmdb.org/t/p/w500/khNVygolU0TxLIDWff5tQlAhZ23.jpg",
        8.2,
        "Rocky's bloodied rise to power puts him against the deadly Adheera and Ramika Sen."
    );
    let m_pathaan = movie!(
        "Pathaan",
        "Action / Thriller",
        "Hindi",
        146,
        "https://image.tmdb.org/t/p/w500/cQOuFy19m0B8kpui7enCthaI8rP.jpg",
        5.9,
        "An exiled spy takes on a lethal mercenary organisation threatening India."
    );
    let m_jawan = movie!(
        "Jawan",
        "Action / Thriller",
        "Hindi",
        169,
        "https://image.tmdb.org/t/p/w500/uRd0SU6vt84DXobyQ5AI5OG7Mh4.jpg",
        6.5,
        "A prison warden recruits inmates to expose corrupt politicians and unfinished business."
    );
    let m_inception = movie!(
        "Inception",
        "Sci-Fi / Thriller",
        "English",
        148,
        "https://image.tmdb.org/t/p/w500/oYuLEt3zVCKq57qu2F8dT7NIa6f.jpg",
        8.8,
        "A thief who enters people's dreams to steal secrets is tasked with planting an idea instead."
    );
    let m_interstellar = movie!(
        "Interstellar",
        "Sci-Fi / Drama",
        "English",
        169,
        "https://image.tmdb.org/t/p/w500/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
        8.6,
        "A team of explorers travel through a wormhole in space to ensure humanity's survival."
    );
    let m_joker = movie!(
        "Joker",
        "Crime / Drama",
        "English",
        122,
        "https://image.tmdb.org/t/p/w500/udDclJoHjfjb8Ekgsd4FDteOkCU.jpg",
        8.4,
        "A failed comedian descends into madness and becomes the criminal mastermind Joker."
    );
    let m_rrrr = movie!(
        "RRR",
        "Action / Drama",
        "Telugu",
        182,
        "https://image.tmdb.org/t/p/w500/ljHw5eIMnki3HekwkKwCCHsRSbH.jpg",
        7.8,
        "Two legendary Indian revolutionaries team up to fight British colonial rule."
    );
    let m_kalki = movie!(
        "Kalki 2898 AD",
        "Sci-Fi / Action",
        "Telugu",
        181,
        "https://image.tmdb.org/t/p/w500/zGLHX92Gk96O1DJvLil7ObJTbaL.jpg",
        6.7,
        "In a dystopian future, a reluctant hero must fulfil an ancient prophecy."
    );

    tracing::info!("seeded 14 movies");

    // ── Shows ─────────────────────────────────────────────────────────────────
    let now = chrono::Utc::now();
    let h = chrono::Duration::hours(1);

    macro_rules! row {
        ($r:expr, $n:expr, $t:expr) => {
            RowConfig {
                row: $r.to_string(),
                seats: $n,
                seat_type: $t.to_string(),
            }
        };
    }

    let normal_hall = |seats: u8| -> Vec<RowConfig> {
        vec![
            row!("A", seats as u32, "Standard"),
            row!("B", seats as u32, "Standard"),
            row!("C", seats as u32, "Comfort"),
            row!("D", seats as u32, "Comfort"),
            row!("E", seats as u32, "Comfort"),
            row!("F", seats as u32, "Comfort"),
            row!("G", seats as u32, "Comfort"),
            row!("H", seats as u32, "Recliner"),
            row!("I", seats as u32, "Recliner"),
        ]
    };
    let luxe_hall = |rows: u8, seats: u8| -> Vec<RowConfig> {
        (b'A'..=b'A' + rows - 1)
            .map(|r| row!(std::str::from_utf8(&[r]).unwrap(), seats as u32, "Recliner"))
            .collect()
    };
    let imax_hall = || {
        vec![
            row!("A", 14, "Standard"),
            row!("B", 14, "Standard"),
            row!("C", 16, "Comfort"),
            row!("D", 16, "Comfort"),
            row!("E", 16, "Comfort"),
            row!("F", 18, "Comfort"),
            row!("G", 18, "Comfort"),
            row!("H", 18, "Recliner"),
            row!("I", 18, "Recliner"),
        ]
    };

    let shows: Vec<CreateShowRequest> = vec![
        // Avengers — 3 shows across 3 venues/timeslots
        CreateShowRequest {
            show_name: "Avengers: Endgame".into(),
            theatre_name: "PVR Nexus".into(),
            screen_number: 1,
            start_time: now + h,
            end_time: now + h * 4,
            price_per_seat: 200.0,
            movie_id: Some(m_avengers.movie_id.clone()),
            venue_id: Some(v_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        CreateShowRequest {
            show_name: "Avengers: Endgame".into(),
            theatre_name: "IMAX Central".into(),
            screen_number: 1,
            start_time: now + h * 5,
            end_time: now + h * 8,
            price_per_seat: 350.0,
            movie_id: Some(m_avengers.movie_id.clone()),
            venue_id: Some(v_imax.venue_id.clone()),
            seat_layout: SeatLayoutRequest { rows: imax_hall() },
        },
        CreateShowRequest {
            show_name: "Avengers: Endgame".into(),
            theatre_name: "PVR Mumbai".into(),
            screen_number: 2,
            start_time: now + h * 9,
            end_time: now + h * 12,
            price_per_seat: 220.0,
            movie_id: Some(m_avengers.movie_id.clone()),
            venue_id: Some(v_pvr_m.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // Dune — 3 shows
        CreateShowRequest {
            show_name: "Dune: Part Two".into(),
            theatre_name: "IMAX Central".into(),
            screen_number: 2,
            start_time: now + h * 2,
            end_time: now + h * 5,
            price_per_seat: 350.0,
            movie_id: Some(m_dune.movie_id.clone()),
            venue_id: Some(v_imax.venue_id.clone()),
            seat_layout: SeatLayoutRequest { rows: imax_hall() },
        },
        CreateShowRequest {
            show_name: "Dune: Part Two".into(),
            theatre_name: "PVR Delhi".into(),
            screen_number: 1,
            start_time: now + h * 6,
            end_time: now + h * 9,
            price_per_seat: 300.0,
            movie_id: Some(m_dune.movie_id.clone()),
            venue_id: Some(v_pvr_d.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(11),
            },
        },
        CreateShowRequest {
            show_name: "Dune: Part Two".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 1,
            start_time: now + h * 10,
            end_time: now + h * 13,
            price_per_seat: 280.0,
            movie_id: Some(m_dune.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // Spider-Man — 3 shows
        CreateShowRequest {
            show_name: "Spider-Man: No Way Home".into(),
            theatre_name: "Gold Class Cinemas".into(),
            screen_number: 1,
            start_time: now + h * 3,
            end_time: now + h * 6,
            price_per_seat: 500.0,
            movie_id: Some(m_spider.movie_id.clone()),
            venue_id: Some(v_gold.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: luxe_hall(4, 8),
            },
        },
        CreateShowRequest {
            show_name: "Spider-Man: No Way Home".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 3,
            start_time: now + h * 7,
            end_time: now + h * 10,
            price_per_seat: 280.0,
            movie_id: Some(m_spider.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        CreateShowRequest {
            show_name: "Spider-Man: No Way Home".into(),
            theatre_name: "Wave Cinemas".into(),
            screen_number: 2,
            start_time: now + h * 11,
            end_time: now + h * 14,
            price_per_seat: 200.0,
            movie_id: Some(m_spider.movie_id.clone()),
            venue_id: Some(v_wave.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // Grand Budapest Hotel — 3 shows
        CreateShowRequest {
            show_name: "The Grand Budapest Hotel".into(),
            theatre_name: "Retro Cinema".into(),
            screen_number: 1,
            start_time: now + h * 4,
            end_time: now + h * 6,
            price_per_seat: 150.0,
            movie_id: Some(m_budapest.movie_id.clone()),
            venue_id: Some(v_retro.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(15),
            },
        },
        CreateShowRequest {
            show_name: "The Grand Budapest Hotel".into(),
            theatre_name: "Wave Cinemas".into(),
            screen_number: 1,
            start_time: now + h * 8,
            end_time: now + h * 10,
            price_per_seat: 180.0,
            movie_id: Some(m_budapest.movie_id.clone()),
            venue_id: Some(v_wave.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "The Grand Budapest Hotel".into(),
            theatre_name: "PVR Delhi".into(),
            screen_number: 3,
            start_time: now + h * 12,
            end_time: now + h * 14,
            price_per_seat: 160.0,
            movie_id: Some(m_budapest.movie_id.clone()),
            venue_id: Some(v_pvr_d.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        // Oppenheimer — 3 shows
        CreateShowRequest {
            show_name: "Oppenheimer".into(),
            theatre_name: "PVR Nexus".into(),
            screen_number: 2,
            start_time: now + h * 2,
            end_time: now + h * 5,
            price_per_seat: 250.0,
            movie_id: Some(m_oppenheimer.movie_id.clone()),
            venue_id: Some(v_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Oppenheimer".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 2,
            start_time: now + h * 6,
            end_time: now + h * 9,
            price_per_seat: 220.0,
            movie_id: Some(m_oppenheimer.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Oppenheimer".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 1,
            start_time: now + h * 14,
            end_time: now + h * 17,
            price_per_seat: 300.0,
            movie_id: Some(m_oppenheimer.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // Barbie — 3 shows
        CreateShowRequest {
            show_name: "Barbie".into(),
            theatre_name: "Gold Class Cinemas".into(),
            screen_number: 2,
            start_time: now + h * 1,
            end_time: now + h * 3,
            price_per_seat: 450.0,
            movie_id: Some(m_barbie.movie_id.clone()),
            venue_id: Some(v_gold.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: vec![row!("A", 8, "Recliner"), row!("B", 8, "Recliner")],
            },
        },
        CreateShowRequest {
            show_name: "Barbie".into(),
            theatre_name: "PVR Mumbai".into(),
            screen_number: 3,
            start_time: now + h * 5,
            end_time: now + h * 7,
            price_per_seat: 200.0,
            movie_id: Some(m_barbie.movie_id.clone()),
            venue_id: Some(v_pvr_m.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        CreateShowRequest {
            show_name: "Barbie".into(),
            theatre_name: "Wave Cinemas".into(),
            screen_number: 3,
            start_time: now + h * 9,
            end_time: now + h * 11,
            price_per_seat: 170.0,
            movie_id: Some(m_barbie.movie_id.clone()),
            venue_id: Some(v_wave.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // KGF 2 — 3 shows
        CreateShowRequest {
            show_name: "KGF: Chapter 2".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 2,
            start_time: now + h * 3,
            end_time: now + h * 6,
            price_per_seat: 250.0,
            movie_id: Some(m_kgf.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "KGF: Chapter 2".into(),
            theatre_name: "PVR Nexus".into(),
            screen_number: 3,
            start_time: now + h * 7,
            end_time: now + h * 10,
            price_per_seat: 220.0,
            movie_id: Some(m_kgf.movie_id.clone()),
            venue_id: Some(v_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        CreateShowRequest {
            show_name: "KGF: Chapter 2".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 3,
            start_time: now + h * 11,
            end_time: now + h * 14,
            price_per_seat: 200.0,
            movie_id: Some(m_kgf.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // Pathaan — 3 shows
        CreateShowRequest {
            show_name: "Pathaan".into(),
            theatre_name: "PVR Delhi".into(),
            screen_number: 2,
            start_time: now + h * 2,
            end_time: now + h * 5,
            price_per_seat: 210.0,
            movie_id: Some(m_pathaan.movie_id.clone()),
            venue_id: Some(v_pvr_d.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Pathaan".into(),
            theatre_name: "PVR Mumbai".into(),
            screen_number: 1,
            start_time: now + h * 6,
            end_time: now + h * 9,
            price_per_seat: 230.0,
            movie_id: Some(m_pathaan.movie_id.clone()),
            venue_id: Some(v_pvr_m.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Pathaan".into(),
            theatre_name: "Wave Cinemas".into(),
            screen_number: 1,
            start_time: now + h * 10,
            end_time: now + h * 13,
            price_per_seat: 180.0,
            movie_id: Some(m_pathaan.movie_id.clone()),
            venue_id: Some(v_wave.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        // Jawan — 3 shows
        CreateShowRequest {
            show_name: "Jawan".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 4,
            start_time: now + h * 1,
            end_time: now + h * 4,
            price_per_seat: 240.0,
            movie_id: Some(m_jawan.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Jawan".into(),
            theatre_name: "PVR Nexus".into(),
            screen_number: 4,
            start_time: now + h * 5,
            end_time: now + h * 8,
            price_per_seat: 220.0,
            movie_id: Some(m_jawan.movie_id.clone()),
            venue_id: Some(v_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Jawan".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 4,
            start_time: now + h * 9,
            end_time: now + h * 12,
            price_per_seat: 200.0,
            movie_id: Some(m_jawan.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        // Inception — 3 shows
        CreateShowRequest {
            show_name: "Inception".into(),
            theatre_name: "Retro Cinema".into(),
            screen_number: 1,
            start_time: now + h * 2,
            end_time: now + h * 5,
            price_per_seat: 160.0,
            movie_id: Some(m_inception.movie_id.clone()),
            venue_id: Some(v_retro.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(15),
            },
        },
        CreateShowRequest {
            show_name: "Inception".into(),
            theatre_name: "PVR Delhi".into(),
            screen_number: 4,
            start_time: now + h * 7,
            end_time: now + h * 10,
            price_per_seat: 200.0,
            movie_id: Some(m_inception.movie_id.clone()),
            venue_id: Some(v_pvr_d.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Inception".into(),
            theatre_name: "IMAX Central".into(),
            screen_number: 3,
            start_time: now + h * 13,
            end_time: now + h * 16,
            price_per_seat: 350.0,
            movie_id: Some(m_inception.movie_id.clone()),
            venue_id: Some(v_imax.venue_id.clone()),
            seat_layout: SeatLayoutRequest { rows: imax_hall() },
        },
        // Interstellar — 3 shows
        CreateShowRequest {
            show_name: "Interstellar".into(),
            theatre_name: "IMAX Central".into(),
            screen_number: 2,
            start_time: now + h * 3,
            end_time: now + h * 6,
            price_per_seat: 380.0,
            movie_id: Some(m_interstellar.movie_id.clone()),
            venue_id: Some(v_imax.venue_id.clone()),
            seat_layout: SeatLayoutRequest { rows: imax_hall() },
        },
        CreateShowRequest {
            show_name: "Interstellar".into(),
            theatre_name: "PVR Mumbai".into(),
            screen_number: 4,
            start_time: now + h * 8,
            end_time: now + h * 11,
            price_per_seat: 250.0,
            movie_id: Some(m_interstellar.movie_id.clone()),
            venue_id: Some(v_pvr_m.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Interstellar".into(),
            theatre_name: "Gold Class Cinemas".into(),
            screen_number: 1,
            start_time: now + h * 15,
            end_time: now + h * 18,
            price_per_seat: 550.0,
            movie_id: Some(m_interstellar.movie_id.clone()),
            venue_id: Some(v_gold.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: luxe_hall(4, 6),
            },
        },
        // Joker — 3 shows
        CreateShowRequest {
            show_name: "Joker".into(),
            theatre_name: "Retro Cinema".into(),
            screen_number: 1,
            start_time: now + h * 6,
            end_time: now + h * 8,
            price_per_seat: 140.0,
            movie_id: Some(m_joker.movie_id.clone()),
            venue_id: Some(v_retro.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(15),
            },
        },
        CreateShowRequest {
            show_name: "Joker".into(),
            theatre_name: "Wave Cinemas".into(),
            screen_number: 2,
            start_time: now + h * 10,
            end_time: now + h * 12,
            price_per_seat: 180.0,
            movie_id: Some(m_joker.movie_id.clone()),
            venue_id: Some(v_wave.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(10),
            },
        },
        CreateShowRequest {
            show_name: "Joker".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 5,
            start_time: now + h * 14,
            end_time: now + h * 16,
            price_per_seat: 200.0,
            movie_id: Some(m_joker.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        // RRR — 3 shows
        CreateShowRequest {
            show_name: "RRR".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 1,
            start_time: now + h * 1,
            end_time: now + h * 4,
            price_per_seat: 220.0,
            movie_id: Some(m_rrrr.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "RRR".into(),
            theatre_name: "PVR Nexus".into(),
            screen_number: 5,
            start_time: now + h * 5,
            end_time: now + h * 8,
            price_per_seat: 200.0,
            movie_id: Some(m_rrrr.movie_id.clone()),
            venue_id: Some(v_pvr.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "RRR".into(),
            theatre_name: "Cinepolis Mumbai".into(),
            screen_number: 6,
            start_time: now + h * 9,
            end_time: now + h * 12,
            price_per_seat: 240.0,
            movie_id: Some(m_rrrr.movie_id.clone()),
            venue_id: Some(v_cinep.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(11),
            },
        },
        // Kalki 2898 AD — 3 shows
        CreateShowRequest {
            show_name: "Kalki 2898 AD".into(),
            theatre_name: "INOX Hyderabad".into(),
            screen_number: 2,
            start_time: now + h * 3,
            end_time: now + h * 6,
            price_per_seat: 250.0,
            movie_id: Some(m_kalki.movie_id.clone()),
            venue_id: Some(v_inox.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Kalki 2898 AD".into(),
            theatre_name: "PVR Mumbai".into(),
            screen_number: 2,
            start_time: now + h * 7,
            end_time: now + h * 10,
            price_per_seat: 230.0,
            movie_id: Some(m_kalki.movie_id.clone()),
            venue_id: Some(v_pvr_m.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
        CreateShowRequest {
            show_name: "Kalki 2898 AD".into(),
            theatre_name: "PVR Delhi".into(),
            screen_number: 3,
            start_time: now + h * 11,
            end_time: now + h * 14,
            price_per_seat: 210.0,
            movie_id: Some(m_kalki.movie_id.clone()),
            venue_id: Some(v_pvr_d.venue_id.clone()),
            seat_layout: SeatLayoutRequest {
                rows: normal_hall(12),
            },
        },
    ];

    for show in shows {
        match show_svc.create_show(show).await {
            Ok((s, seats)) => {
                tracing::info!(show_id = %s.show_id, seat_count = seats.len(), "seeded show");
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to seed show");
            }
        }
    }
    tracing::info!("seed complete");
}
