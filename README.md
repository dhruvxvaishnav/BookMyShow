# BookMyShow — Rust Backend

> A production-grade movie seat booking backend built in Rust.  
> This project is how I taught myself real-world backend engineering, system design, and concurrent programming.

---

## The Story

I started this project because I wanted to understand how systems like BookMyShow actually work under the hood.

Not the UI. Not the app. The **hard part** — what happens in the milliseconds between two users clicking the same seat at the same time? Who gets it? How does the server guarantee only one person wins? What happens to the loser's payment?

These questions pulled me into a rabbit hole of concurrent systems, distributed locking, state machines, and async Rust. This repo is the result. Every design decision in here maps to something I had to learn, break, fix, and understand from first principles.

---

## What This Project Taught Me

### 1. Race Conditions Are Invisible Until They're Catastrophic

The core problem: two users simultaneously select seat A1. Both see it as `Available`. Both send their requests. Without synchronisation, **both succeed** — you've now double-booked the same seat.

This is a race condition. It doesn't happen in testing. It happens at 10,000 concurrent users on premiere night.

The fix I implemented: a **per-show `RwLock`**. Before any seat lock acquisition, the service acquires an exclusive write guard scoped to that `show_id`. Only one lock operation per show can proceed at a time. I learned that the lock granularity matters enormously — a global mutex would serialise all shows; per-show granularity means Show A's lock contention never affects Show B.

```
User A ──────► acquire show_mutex ──► check seat ──► lock seat ──► release mutex
User B ──────► waiting...............► acquire show_mutex ──► seat already Locked ──► 409
```

### 2. Double-Checked Locking Is a Real Pattern

Even inside the critical section, I do a second database read. Why?

Because the first check happened **before** the mutex was acquired. By the time we're inside the lock, another goroutine might have already changed state. Reading once before, and once inside the critical section, is called **double-checked locking** — and it's the only way to be certain.

```rust
// BEFORE acquiring mutex (fast path, no contention)
let seats = repo.find_by_ids(&seat_ids).await?;

// Acquire per-show mutex
let _guard = show_lock.write().await;

// INSIDE critical section: re-read to catch any changes since the pre-check
let current_seats = repo.find_by_ids(&seat_ids).await?;
for seat in &current_seats {
    if seat.status != SeatStatus::Available {
        return Err(AppError::SeatsUnavailable(...));
    }
}
```

### 3. State Machines Make Business Logic Explicit

A booking goes through a well-defined lifecycle. Trying to track this with boolean flags (`is_paid`, `is_cancelled`, `is_confirmed`) leads to impossible states. Rust enums make invalid states unrepresentable:

```
Pending ──► PaymentPending ──► Success
   │               │
   ▼               ▼
Cancelled     PaymentFailed
   │
   ▼
 Expired
```

Every service method that mutates a booking first checks the current status and rejects transitions that don't belong in the state machine. This eliminated an entire class of bugs.

### 4. Background Tasks Are How You Build Time-Aware Systems

A seat lock has a TTL (5 minutes by default). When it expires, the seat must go back to `Available` automatically — even if the user just closed their browser and walked away.

I learned that you can't rely on request-driven cleanup for this. You need a **background task** — an async loop that wakes up every 10 seconds, finds expired locks, and releases them:

```rust
tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(10));
    loop {
        ticker.tick().await;
        lock_svc.process_expired_locks().await?;
    }
});
```

This is also how payment timeouts work — a second background task runs every 30 seconds and expires payments that never received a gateway callback.

### 5. The Repository Pattern Makes Code Testable

Every data access goes through a trait:

```rust
pub trait SeatRepository: Send + Sync {
    async fn find_by_id(&self, seat_id: &str) -> Result<Option<Seat>, AppError>;
    async fn lock_seat(...) -> Result<Seat, AppError>;
    // ...
}
```

The service layer depends on **the trait, not the implementation**. In tests, I swap in `InMemoryRepository`. In production, the same code will work with `PostgresRepository`. This is the Dependency Inversion Principle — high-level policy shouldn't depend on low-level details.

### 6. Idempotency Is About Making Retries Safe

Payment is the most dangerous operation. Networks fail. Users double-click. Servers retry. If `initiate_payment` isn't idempotent, a user gets charged twice.

My solution: an `Idempotency-Key` header. The first call creates the payment; subsequent calls with the same key return the same payment. The key is stored with the payment record and checked on every call:

```rust
if let Some(key) = &idempotency_key
    && let Some(existing) = repo.find_by_idempotency_key(key).await?
{
    return Ok(existing); // safe to call multiple times
}
```

### 7. Error Handling as Documentation

Rust's `Result<T, E>` forces you to think about every failure mode. I modelled every error as a typed enum variant:

```rust
AppError::SeatsUnavailable(Vec<String>)   // 409 — with which seats
AppError::LockExpired(String)             // 410 — with which lock
AppError::RateLimitExceeded              // 429
AppError::PaymentMismatch { expected, actual } // 422
```

Each variant maps to an HTTP status code. The error response includes a machine-readable `code` string so clients can handle specific errors without string-matching messages.

### 8. Rate Limiting Protects You From Yourself

A user who spams `POST /shows/:id/seats/lock` can starve other users during a high-demand window. A sliding-window rate limiter per user-per-endpoint prevents this. I implemented it with a `VecDeque<Instant>` — push on each request, prune entries older than 60 seconds, reject if count exceeds the limit.

### 9. The Queue System: Fairness Under Load

Without a queue, concurrent lock requests race in an uncontrolled way. With hundreds of users hitting the same show simultaneously, the "winner" is whoever's request arrived first by network latency — essentially random.

The per-show queue fixes this. Users join the queue (`POST /shows/:id/queue/join`), get a position, and a background processor grants locks in order. Users poll their status. This is how real ticketing systems handle surge traffic.

### 10. Partial Failures Need Compensation Logs

When a payment succeeds but a seat confirmation fails (a race I discovered could theoretically happen), I needed a way to track it. I created `SuccessPartial` as a booking status and a `CompensationLog` table:

```rust
CompensationLog {
    confirmed_seats: Vec<String>,  // seats that made it
    failed_seats: Vec<String>,     // seats that didn't
    failed_amount: f64,            // pro-rata refund candidate
}
```

This is the foundation of a real compensation pattern — you acknowledge the inconsistency, record it durably, and process the refund asynchronously. No data loss, no silent corruption.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  HTTP Layer (Axum)                │
│  /shows  /bookings  /payments  /queue  /admin     │
└─────────────────────┬────────────────────────────┘
                      │ AppState (Arc-shared)
                      ▼
┌──────────────────────────────────────────────────┐
│               Service Layer (Async)               │
│                                                   │
│  SeatLockingService  ←── per-show RwLock map      │
│  BookingService      ←── state machine            │
│  PaymentService      ←── mock gateway + idempotency│
│  ShowService         ←── CRUD + analytics         │
│  QueueService        ←── fair-queue processor     │
└─────────────────────┬────────────────────────────┘
                      │ Repository Traits
                      ▼
┌──────────────────────────────────────────────────┐
│            Repository Layer (Trait-based)         │
│                                                   │
│  InMemoryRepository  (default, dev + tests)       │
│  PostgresRepository  (planned)                    │
└──────────────────────────────────────────────────┘

Background Tasks (Tokio):
  ├── Lock expiration sweep    every 10s
  ├── Payment timeout sweep    every 30s
  └── Queue processor          every 500ms
```

### Cargo Workspace Layout

```
backend/
├── crates/
│   ├── common/          # AppConfig, AppError, ApiResponse
│   ├── domain/          # Pure domain structs + enums (no I/O)
│   ├── repository/      # Repository traits (interfaces)
│   ├── repository-inmemory/  # In-memory implementations
│   ├── service/         # All business logic + benches/
│   └── api/             # Axum handlers, routes, main.rs
└── config.toml          # Runtime configuration
```

The dependency direction is strict:

```
api → service → repository → domain
              ↘ common ↙
```

`domain` has zero external dependencies. `service` knows nothing about HTTP. This layering is what makes the system testable and replaceable.

---

## Domain Models

| Struct | Description |
|--------|-------------|
| `Show` | A movie screening with time, theatre, price |
| `Seat` | One seat in a show — status: `Available / Locked / Booked` |
| `SeatLock` | A timed hold on a set of seats for one user |
| `Booking` | A user's intent to purchase seats, with full lifecycle status |
| `Payment` | Payment record linked to a booking, with gateway reference |
| `QueueEntry` | A user's place in the per-show request queue |
| `CompensationLog` | Audit record for partial booking failures |

### Booking Status Flow

```
Pending ──[initiate payment]──► PaymentPending ──[gateway SUCCESS]──► Success
   │                                   │                                  │
   │                                   └───[gateway FAILED]──► PaymentFailed
   │                                   └───[partial seats]──► SuccessPartial
   ├──[user cancels]──► Cancelled
   └──[TTL expires]──► Expired
```

### Seat Status Flow

```
Available ──[lock_seats]──► Locked ──[confirm_booking]──► Booked
                              │
                              └──[release_lock / expire]──► Available
```

---

## API Reference

### Shows

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/admin/shows` | Admin | Create show + auto-generate seats |
| `GET` | `/shows` | Public | List all shows |
| `GET` | `/shows/:id` | Public | Get show details |
| `GET` | `/shows/:id/seats` | Public | Paginated seat layout with status |
| `GET` | `/shows/:id/availability` | Public | Available / locked / booked counts |
| `DELETE` | `/admin/shows/:id` | Admin | Cancel show + refund all bookings |
| `GET` | `/admin/shows/:id/analytics` | Admin | Occupancy rate + revenue |

### Booking & Locking

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/shows/:id/seats/lock` | `X-User-Id` | Lock seats → creates booking |
| `POST` | `/bookings/:id/extend-lock` | `X-User-Id` | Extend lock TTL (max 2×) |
| `DELETE` | `/bookings/:id/lock` | `X-User-Id` | Release lock manually |
| `GET` | `/bookings/:id` | `X-User-Id` | Get booking status |
| `POST` | `/bookings/:id/cancel` | `X-User-Id` | Cancel booking |
| `GET` | `/bookings/user/:uid` | `X-User-Id` | User's booking history |
| `GET` | `/admin/bookings` | Admin | All bookings in system |
| `POST` | `/admin/shows/:sid/seats/:id/override` | Admin | Force-release a locked seat |

### Payment

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/bookings/:id/payment/initiate` | `X-User-Id` | Start payment (idempotent) |
| `POST` | `/payments/callback/:intent_id` | Internal | Gateway callback |
| `GET` | `/payments/:id` | `X-User-Id` | Payment status |
| `POST` | `/payments/:id/refund` | Admin | Issue refund |
| `POST` | `/mock-gateway/pay` | Public | Simulate a payment gateway call |

### Queue

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/shows/:id/queue/join` | `X-User-Id` | Join the seat request queue |
| `GET` | `/queue/:id/status` | `X-User-Id` | Poll queue position + result |
| `DELETE` | `/queue/:id` | `X-User-Id` | Leave queue |

### Standard Response Envelope

```json
{ "success": true, "data": { ... } }
{ "success": false, "error": { "code": "SEATS_UNAVAILABLE", "message": "..." } }
```

---

## Getting Started

### Prerequisites

- Rust 1.78+ (Edition 2024)
- `cargo`

```bash
# Clone
git clone https://github.com/dhruvxvaishnav/BookMyShow
cd BookMyShow/backend

# Build
cargo build --release

# Run (defaults to 0.0.0.0:8080)
cargo run -p api

# Or with env overrides
APP_PORT=9090 LOG_FORMAT=text cargo run -p api
```

### Configuration

All settings are in `backend/config.toml`. Every value can be overridden via environment variable:

| Env Var | Default | Effect |
|---------|---------|--------|
| `APP_PORT` | `8080` | HTTP listen port |
| `APP_HOST` | `0.0.0.0` | HTTP bind address |
| `LOG_LEVEL` | `info` | Tracing log level |
| `LOG_FORMAT` | `json` | `json` or `text` |
| `SEAT_LOCK_TTL_SECS` | `300` | Seat lock duration (seconds) |
| `SEAT_LOCK_MAX_EXTENSIONS` | `2` | Max lock extensions per session |
| `SEAT_LOCK_EXTENSION_SECS` | `120` | Seconds added per extension |
| `SEAT_LOCK_GRACE_PERIOD_SECS` | `30` | Buffer after expiry before release |
| `ADMIN_TOKEN` | `admin-secret` | `X-Admin-Token` header value |

### Quick Tour with curl

```bash
# 1. Create a show (admin)
curl -X POST http://localhost:8080/admin/shows \
  -H "X-Admin-Token: admin-secret" \
  -H "Content-Type: application/json" \
  -d '{
    "show_name": "Avengers: Endgame",
    "theatre_name": "PVR Nexus",
    "screen_number": 1,
    "start_time": 1777000000,
    "end_time": 1777010000,
    "price_per_seat": 250.0,
    "seat_layout": {
      "rows": [
        { "row": "A", "seats": 10, "seat_type": "premium" },
        { "row": "B", "seats": 10, "seat_type": "standard" },
        { "row": "C", "seats": 10, "seat_type": "recliner" }
      ]
    }
  }'

# 2. List seats (use the show_id from above)
curl http://localhost:8080/shows/<show_id>/seats

# 3. Lock seats
curl -X POST http://localhost:8080/shows/<show_id>/seats/lock \
  -H "X-User-Id: user-test-001" \
  -H "Content-Type: application/json" \
  -d '{ "seat_ids": ["<seat_id_1>", "<seat_id_2>"] }'

# 4. Initiate payment
curl -X POST http://localhost:8080/bookings/<booking_id>/payment/initiate \
  -H "X-User-Id: user-test-001"

# 5. Simulate gateway callback (success)
curl -X POST "http://localhost:8080/payments/callback/<payment_intent_id>?status=SUCCESS"

# 6. Check booking status
curl http://localhost:8080/bookings/<booking_id> \
  -H "X-User-Id: user-test-001"
```

---

## Tests

```bash
# All tests
cargo test

# A specific suite
cargo test -p service

# With output
cargo test -- --nocapture
```

The test suite covers:
- Concurrent seat locking (50 goroutines racing for 1 seat — exactly 1 wins)
- Lock extension and max-extension enforcement
- Full booking flow: lock → pay → confirm
- Payment failure → seat release
- Payment idempotency
- Rate limiting
- Admin operations

---

## Benchmarks

```bash
# Run all benchmarks (generates HTML report in target/criterion/)
cargo bench -p service

# Run a specific benchmark group
cargo bench -p service --bench seat_locking
cargo bench -p service --bench booking_flow

# Open the HTML report
open target/criterion/report/index.html
```

### What's Being Measured

| Benchmark | PRD Target | What it tests |
|-----------|-----------|---------------|
| `lock_single_seat` | < 50ms p99 | Single seat lock hot path |
| `lock_ten_seats` | < 200ms p99 | Maximum seat count lock |
| `lock_n_seats/1..10` | — | Lock time vs seat count |
| `lock_release_roundtrip` | — | Full lock + release cycle |
| `availability_query_40_seats` | < 20ms p99 | Seat status scan |
| `lock_extension` | — | TTL extension hot path |
| `concurrent_lock_contention/N` | — | Contention at N=2,5,10,20 users |
| `end_to_end_booking_flow` | < 500ms | lock → pay → confirm |
| `booking_confirmation` | < 100ms p99 | Payment-to-booked transition |
| `payment_callback_processing` | < 500ms p99 | Gateway callback path |
| `expired_lock_sweep_20_active` | — | Background task overhead |

---

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Language | Rust 2024 | Memory safety, fearless concurrency, zero-cost abstractions |
| Async runtime | Tokio | Industry-standard async executor |
| HTTP framework | Axum 0.7 | Ergonomic, tower-compatible, zero-copy routing |
| Serialisation | Serde + serde_json | De-facto standard, derive macros |
| Logging | tracing + tracing-subscriber | Structured JSON logs, spans, async-native |
| Configuration | config + TOML | File + env-var layered config |
| Benchmarking | criterion 0.5 | Statistical benchmarks, HTML reports |
| IDs | uuid v4 | Globally unique, no coordination needed |
| Time | chrono | Timezone-aware timestamps |

---

## System Design Concepts Covered

If you're using this project to learn, here's a map of what's where:

| Concept | Where to look |
|---------|--------------|
| Race condition prevention | `seat_locking_service.rs` → `lock_seats()` |
| Per-resource mutex | `show_locks: HashMap<String, Arc<RwLock<()>>>` |
| Double-checked locking | `lock_seats()` — two `find_by_ids` calls |
| State machine | `booking_status.rs`, `booking_service.rs` |
| Background task pattern | `main.rs` — three `tokio::spawn` loops |
| Repository pattern | `repository/src/*.rs` traits |
| Dependency injection | `AppState` in `state.rs` |
| Idempotency | `payment_service.rs` → `initiate_payment()` |
| TTL + grace period | `seat_lock.rs` → `is_active()`, `is_expired()` |
| Compensation log | `compensation_log.rs`, `booking_service.rs` |
| Rate limiting (sliding window) | `rate_limiter.rs` |
| Fair queue | `queue_service.rs`, `queue_service.rs` → `process_next()` |
| Partial failure handling | `BookingStatus::SuccessPartial` |
| Error modelling | `common/src/error.rs` |
| HTTP error mapping | `impl_from_response.rs` |
| JSON structured logging | `main.rs` — `.json()` layer |

---

## What's Next

- [ ] **Postgres repository** — swap the in-memory layer for real persistence
- [x] **Docker + Docker Compose** — one-command deployment
- [ ] **Input validation** — `validator` crate on all HTTP request bodies
- [ ] **Tests for admin endpoints** — analytics, override, bulk bookings
- [ ] **PostgreSQL transactions** — atomic seat locking at the DB level
- [x] **JWT authentication** — replace the `X-User-Id` header
- [x] **Metrics & Tracing** — Prometheus counters for lock contention, booking rate, error rate and OpenTelemetry tracing
- [ ] **`validator` crate** — validate `show_name`, `email`, `seat_ids` at the HTTP boundary

---

## License

MIT
