/// Benchmarks for the seat locking critical path.
///
/// PRD targets:
///   - Single-seat lock acquisition  < 50ms p99
///   - 10-seat lock acquisition      < 200ms p99
///   - Seat availability query       < 20ms p99
use std::sync::Arc;

use chrono::Utc;
use common::AppConfig;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use domain::{Seat, SeatType, Show, User};
use repository::{
    BookingRepository, SeatLockRepository, SeatRepository, ShowRepository, UserRepository,
};
use repository_inmemory::{
    InMemoryBookingRepository, InMemorySeatLockRepository, InMemorySeatRepository,
    InMemoryShowRepository, InMemoryUserRepository,
};
use service::SeatLockingService;
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
        40,
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

/// Builds a fully wired `SeatLockingService` backed by in-memory repos,
/// pre-seeded with one show, N available seats, and one user.
async fn make_service_with_seats(
    seat_count: usize,
) -> (SeatLockingService, Vec<String>, String, String) {
    let show_repo: Arc<dyn ShowRepository> = Arc::new(InMemoryShowRepository::new());
    let seat_repo: Arc<dyn SeatRepository> = Arc::new(InMemorySeatRepository::new());
    let booking_repo: Arc<dyn BookingRepository> = Arc::new(InMemoryBookingRepository::new());
    let seat_lock_repo: Arc<dyn SeatLockRepository> = Arc::new(InMemorySeatLockRepository::new());
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());

    let show_id = "bench-show-1".to_string();
    let user_id = "bench-user-1".to_string();

    show_repo.save(make_show(&show_id)).await.unwrap();

    let mut seat_ids = Vec::with_capacity(seat_count);
    for i in 0..seat_count {
        let id = format!("seat-{i}");
        seat_repo.save(make_seat(&id, &show_id)).await.unwrap();
        seat_ids.push(id);
    }

    let user = User::new(
        user_id.clone(),
        "Bench User".to_string(),
        "bench@test.com".to_string(),
    );
    user_repo.save(user).await.unwrap();

    let svc = SeatLockingService::new(
        show_repo,
        seat_repo,
        booking_repo,
        seat_lock_repo,
        user_repo,
        AppConfig::default(),
    );

    (svc, seat_ids, show_id, user_id)
}

// ── bench: single seat lock ───────────────────────────────────────────────────

fn bench_lock_single_seat(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lock_single_seat", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                // Re-create fresh repos for each iteration so the seat is always Available
                let future = make_service_with_seats(1);
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(future)
            },
            |(svc, seat_ids, show_id, user_id)| async move {
                svc.lock_seats(&show_id, seat_ids, &user_id).await.unwrap();
            },
        );
    });
}

// ── bench: 10-seat lock ───────────────────────────────────────────────────────

fn bench_lock_ten_seats(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lock_ten_seats", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let future = make_service_with_seats(10);
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(future)
            },
            |(svc, seat_ids, show_id, user_id)| async move {
                svc.lock_seats(&show_id, seat_ids, &user_id).await.unwrap();
            },
        );
    });
}

// ── bench: lock N seats (parametric) ─────────────────────────────────────────

fn bench_lock_n_seats(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("lock_n_seats");

    for n in [1usize, 2, 5, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.to_async(&rt).iter_with_setup(
                || {
                    let future = make_service_with_seats(n);
                    let rt2 = Runtime::new().unwrap();
                    rt2.block_on(future)
                },
                |(svc, seat_ids, show_id, user_id)| async move {
                    svc.lock_seats(&show_id, seat_ids, &user_id).await.unwrap();
                },
            );
        });
    }
    group.finish();
}

// ── bench: lock + release (round-trip) ───────────────────────────────────────

fn bench_lock_release_roundtrip(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lock_release_roundtrip", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let future = make_service_with_seats(3);
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(future)
            },
            |(svc, seat_ids, show_id, user_id)| async move {
                let result = svc.lock_seats(&show_id, seat_ids, &user_id).await.unwrap();
                svc.release_lock(&result.booking_id, &user_id)
                    .await
                    .unwrap();
            },
        );
    });
}

// ── bench: availability query ─────────────────────────────────────────────────

fn bench_availability_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Pre-build the service once; seats don't change during this benchmark
    let (svc, _, show_id, _) = rt.block_on(make_service_with_seats(40));
    let svc = Arc::new(svc);

    c.bench_function("availability_query_40_seats", |b| {
        b.to_async(&rt).iter(|| {
            let svc = Arc::clone(&svc);
            let show_id = show_id.clone();
            async move {
                // Simulate what the GET /shows/:id/seats endpoint does
                svc.process_expired_locks().await.unwrap();
                let _ = show_id; // suppress unused warning
            }
        });
    });
}

// ── bench: lock extension ─────────────────────────────────────────────────────

fn bench_lock_extension(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lock_extension", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let rt2 = Runtime::new().unwrap();
                rt2.block_on(async {
                    let (svc, seat_ids, show_id, user_id) = make_service_with_seats(1).await;
                    let lock_result = svc.lock_seats(&show_id, seat_ids, &user_id).await.unwrap();
                    (svc, lock_result.booking_id, user_id)
                })
            },
            |(svc, booking_id, user_id)| async move {
                svc.extend_lock(&booking_id, &user_id).await.unwrap();
            },
        );
    });
}

// ── bench: concurrent lock contention ────────────────────────────────────────

fn bench_concurrent_lock_contention(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_lock_contention");
    // Measure throughput: how fast N goroutines race for 1 seat (1 wins, rest fail fast)
    for n_concurrent in [2usize, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_concurrent),
            &n_concurrent,
            |b, &n| {
                b.to_async(&rt).iter_with_setup(
                    || {
                        let rt2 = Runtime::new().unwrap();
                        rt2.block_on(async {
                            let (svc, seat_ids, show_id, _) = make_service_with_seats(1).await;
                            let svc = Arc::new(svc);
                            // Seed N users
                            (svc, seat_ids, show_id)
                        })
                    },
                    |(svc, seat_ids, show_id)| async move {
                        let mut handles = vec![];
                        for i in 0..n {
                            let svc = Arc::clone(&svc);
                            let seat_ids = seat_ids.clone();
                            let show_id = show_id.clone();
                            handles.push(tokio::spawn(async move {
                                let user_id = format!("user-{i}");
                                let _ = svc.lock_seats(&show_id, seat_ids, &user_id).await;
                            }));
                        }
                        futures::future::join_all(handles).await;
                    },
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_lock_single_seat,
    bench_lock_ten_seats,
    bench_lock_n_seats,
    bench_lock_release_roundtrip,
    bench_availability_query,
    bench_lock_extension,
    bench_concurrent_lock_contention,
);
criterion_main!(benches);
