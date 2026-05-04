use std::sync::Arc;

use chrono::Utc;
use common::AppConfig;
use criterion::{Criterion, criterion_group, criterion_main};
use domain::{Seat, SeatType, Show, User};
use repository::{
    BookingRepository, CompensationLogRepository, PaymentRepository, SeatLockRepository,
    SeatRepository, ShowRepository, UserRepository,
};
use repository_inmemory::{
    InMemoryBookingRepository, InMemoryCompensationLogRepository, InMemoryPaymentRepository,
    InMemorySeatLockRepository, InMemorySeatRepository, InMemoryShowRepository,
    InMemoryUserRepository,
};
use service::booking_service::BookingServiceTrait;
use service::payment_service::PaymentServiceTrait;
use service::{BookingService, PaymentService, SeatLockingService};
use tokio::runtime::Runtime;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_show(id: &str) -> Show {
    Show::new(
        id.to_string(),
        "Avengers: Endgame".to_string(),
        "PVR Nexus".to_string(),
        1,
        Utc::now() + chrono::Duration::hours(1),
        Utc::now() + chrono::Duration::hours(4),
        250.0,
        20,
    )
}

fn make_seat(id: &str, show_id: &str) -> Seat {
    Seat::new(
        id.to_string(),
        id.to_string(),
        "A".to_string(),
        SeatType::Standard,
        show_id.to_string(),
    )
}

struct BenchServices {
    locking_svc: Arc<SeatLockingService>,
    booking_svc: Arc<dyn BookingServiceTrait>,
    payment_svc: Arc<dyn PaymentServiceTrait>,
    show_id: String,
    user_id: String,
    seat_ids: Vec<String>,
}

async fn make_bench_services(seat_count: usize) -> BenchServices {
    let show_repo: Arc<dyn ShowRepository> = Arc::new(InMemoryShowRepository::new());
    let seat_repo: Arc<dyn SeatRepository> = Arc::new(InMemorySeatRepository::new());
    let booking_repo: Arc<dyn BookingRepository> = Arc::new(InMemoryBookingRepository::new());
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(InMemoryPaymentRepository::new());
    let seat_lock_repo: Arc<dyn SeatLockRepository> = Arc::new(InMemorySeatLockRepository::new());
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
    let comp_log_repo: Arc<dyn CompensationLogRepository> =
        Arc::new(InMemoryCompensationLogRepository::new());

    let show_id = "bench-show-flow".to_string();
    let user_id = "bench-user-flow".to_string();
    let cfg = AppConfig::default();

    show_repo.save(make_show(&show_id)).await.unwrap();

    let mut seat_ids = Vec::new();
    for i in 0..seat_count {
        let id = format!("flow-seat-{i}");
        seat_repo.save(make_seat(&id, &show_id)).await.unwrap();
        seat_ids.push(id);
    }

    user_repo
        .save(User::new(
            user_id.clone(),
            "Bench User".to_string(),
            "bench@test.com".to_string(),
        ))
        .await
        .unwrap();

    let email_svc = Arc::new(service::EmailService::new(cfg.clone()));

    let locking_svc = Arc::new(SeatLockingService::new(
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
        Arc::clone(&comp_log_repo),
        Arc::clone(&user_repo),
        Arc::clone(&show_repo),
        Arc::clone(&email_svc) as Arc<dyn service::EmailServiceTrait>,
        cfg.clone(),
    ));

    let mut cfg_no_fail = cfg.clone();
    cfg_no_fail.payment.mock_gateway_failure_rate = 0.0;

    let payment_svc: Arc<dyn PaymentServiceTrait> = Arc::new(PaymentService::new(
        Arc::clone(&payment_repo),
        Arc::clone(&booking_repo),
        Arc::clone(&user_repo),
        Arc::clone(&booking_svc) as Arc<dyn BookingServiceTrait>,
        Arc::clone(&email_svc) as Arc<dyn service::EmailServiceTrait>,
        cfg_no_fail,
    ));

    BenchServices {
        locking_svc,
        booking_svc,
        payment_svc,
        show_id,
        user_id,
        seat_ids,
    }
}

// ── bench: booking confirmation ───────────────────────────────────────────────

fn bench_booking_confirmation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("booking_confirmation", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                // Setup: lock seats and initiate payment so confirm can run
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(async {
                    let svcs = make_bench_services(2).await;
                    let lock = svcs
                        .locking_svc
                        .lock_seats(&svcs.show_id, svcs.seat_ids.clone(), &svcs.user_id)
                        .await
                        .unwrap();
                    let payment = svcs
                        .payment_svc
                        .initiate_payment(&lock.booking_id, &svcs.user_id, None)
                        .await
                        .unwrap();
                    // Manually mark payment as succeeded so confirm_booking can run
                    (svcs, lock.booking_id, payment.payment_id)
                })
            },
            |(svcs, booking_id, payment_id)| async move {
                svcs.booking_svc
                    .confirm_booking(&booking_id, &payment_id)
                    .await
                    .unwrap();
            },
        );
    });
}

fn bench_end_to_end_flow(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("end_to_end_booking_flow", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(make_bench_services(3))
            },
            |svcs| async move {
                // 1. Lock seats
                let lock = svcs
                    .locking_svc
                    .lock_seats(&svcs.show_id, svcs.seat_ids.clone(), &svcs.user_id)
                    .await
                    .unwrap();

                // 2. Initiate payment
                let payment = svcs
                    .payment_svc
                    .initiate_payment(&lock.booking_id, &svcs.user_id, None)
                    .await
                    .unwrap();

                // 3. Mock gateway callback (always success — failure_rate = 0.0)
                svcs.payment_svc
                    .payment_callback(&payment.payment_intent_id, "SUCCESS", None)
                    .await
                    .unwrap();
            },
        );
    });
}

// ── bench: payment callback processing ───────────────────────────────────────

fn bench_payment_callback(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("payment_callback_processing", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(async {
                    let svcs = make_bench_services(1).await;
                    let lock = svcs
                        .locking_svc
                        .lock_seats(&svcs.show_id, svcs.seat_ids.clone(), &svcs.user_id)
                        .await
                        .unwrap();
                    let payment = svcs
                        .payment_svc
                        .initiate_payment(&lock.booking_id, &svcs.user_id, None)
                        .await
                        .unwrap();
                    (svcs, payment.payment_intent_id)
                })
            },
            |(svcs, payment_intent_id)| async move {
                svcs.payment_svc
                    .payment_callback(&payment_intent_id, "SUCCESS", None)
                    .await
                    .unwrap();
            },
        );
    });
}

// ── bench: lock expiration sweep ─────────────────────────────────────────────

fn bench_expired_lock_sweep(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Pre-seed 20 locks (all still active — measures sweep overhead without expiry work)
    let lock_svc = rt.block_on(async {
        let svcs = make_bench_services(20).await;
        let lock_svc = Arc::clone(&svcs.locking_svc);
        let _ = svcs
            .locking_svc
            .lock_seats(&svcs.show_id, svcs.seat_ids.clone(), &svcs.user_id)
            .await;
        lock_svc
    });

    c.bench_function("expired_lock_sweep_20_active", |b| {
        b.to_async(&rt).iter(|| {
            let lock_svc = Arc::clone(&lock_svc);
            async move {
                lock_svc.process_expired_locks().await.unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    bench_booking_confirmation,
    bench_end_to_end_flow,
    bench_payment_callback,
    bench_expired_lock_sweep,
);
criterion_main!(benches);
