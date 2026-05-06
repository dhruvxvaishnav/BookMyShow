# BookMyShow — Rust Backend Rewrite
## Product Requirements Document (PRD)

> **Version:** 1.0
> **Date:** 2026-04-28
> **Status:** Draft
> **Language:** Rust (Edition 2024)

---

## Table of Contents

1. [Vision & Goals](#1-vision--goals)
2. [Non-Goals](#2-non-goals)
3. [High-Level Architecture](#3-high-level-architecture)
4. [Domain Models](#4-domain-models)
5. [Core Feature: Seat Inventory Management](#5-core-feature-seat-inventory-management)
6. [Core Feature: Seat Locking System](#6-core-feature-seat-locking-system)
7. [Core Feature: Request Queue & Concurrency Control](#7-core-feature-request-queue--concurrency-control)
8. [Core Feature: Booking Flow](#8-core-feature-booking-flow)
9. [Core Feature: Payment Processing](#9-core-feature-payment-processing)
10. [Core Feature: HTTP API Layer](#10-core-feature-http-api-layer)
11. [Core Feature: Admin / Show Management](#11-core-feature-admin--show-management)
12. [Concurrency & Thread Safety](#12-concurrency--thread-safety)
13. [Error Handling Strategy](#13-error-handling-strategy)
14. [Data Persistence Layer](#14-data-persistence-layer)
15. [Configuration & Environment](#15-configuration--environment)
16. [Non-Functional Requirements](#16-non-functional-requirements)
17. [Phased Implementation Roadmap](#17-phased-implementation-roadmap)
18. [Out of Scope](#18-out-of-scope)
19. [Glossary](#19-glossary)
20. [Revision History](#20-revision-history)

---

## 1. Vision & Goals

### 1.1 Product Vision

A production-ready, high-performance movie seat booking backend system written in Rust, designed to handle concurrent seat selection requests safely. The system enables users to browse shows, hold seats temporarily, complete payment, and confirm bookings — all with strong guarantees around seat availability, race-condition prevention, and graceful timeout handling.

### 1.2 Success Criteria

| # | Criterion | Measurement |
|---|-----------|-------------|
| 1 | Two simultaneous users can **never** book the same seat | Integration test with concurrent HTTP requests |
| 2 | A seat lock expires after exactly the configured TTL | Unit test verifies lock expiration |
| 3 | Expired locks automatically release seats back to `Available` | Background task / async timer |
| 4 | Payment timeout also releases the seat lock | Timer triggers lock release on expiry |
| 5 | All HTTP endpoints return proper status codes and JSON bodies | API integration tests |
| 6 | System compiles with zero warnings in Rust 2024 edition | `cargo build --release` |
| 7 | Core booking flow completes end-to-end in < 500ms (in-memory) | Benchmark test |

### 1.3 Current State vs Target State

| Aspect | Current | Target |
|--------|---------|--------|
| Language | Rust | Rust |
| Build Edition | 2024 | 2024 |
| Storage | In-memory HashMap | In-memory + pluggable DB |
| HTTP API | None (main.rs empty) | Axum-based REST API |
| Seat Locking | Defined but unused | Fully implemented with TTL |
| Queue System | None | Token-based fair-queue |
| Concurrency Safety | None (no Mutex) | Per-show Mutex locks |
| Error Handling | Panic-based | `Result<T, AppError>` |
| Async/Await | Synchronous only | Full async runtime (Tokio) |
| Testing | None | Unit + Integration tests |

---

## 2. Non-Goals

- Mobile or web frontend (pure backend)
- Multi-theatre / multi-city support (single instance, single theatre)
- Real payment gateway integration (mocked/simulated only)
- Admin dashboard UI
- Email / SMS notifications
- User authentication / JWT (auth is out of scope; user_id is passed as header/input)
- Distributed deployment (single-node only)

---

## 3. High-Level Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                      HTTP Layer (Axum)                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ Show API │ │ Seat API │ │ Booking  │ │ Payment Callback │  │
│  │          │ │          │ │   API    │ │      API         │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬─────────┘  │
└───────┼────────────┼────────────┼────────────────┼────────────┘
        │            │            │                │
        ▼            ▼            ▼                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Service Layer (Async)                     │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────┐  │
│  │ ShowService    │  │ SeatLockService│  │ BookingService│  │
│  │                │  │                │  │               │  │
│  │ - list_shows   │  │ - lock_seats   │  │ - create_booking │
│  │ - get_show     │  │ - unlock_seats │  │ - confirm     │  │
│  │ - create_show  │  │ - extend_lock  │  │ - cancel      │  │
│  │                │  │ - expire_locks │  │ - get_status  │  │
│  └────────┬───────┘  └───────┬────────┘  └───────┬───────┘  │
│           │                  │                   │          │
│  ┌────────┴──────────────────┴───────────────────┴────────┐ │
│  │              PaymentService                            │ │
│  │  - initiate_payment  - payment_callback  - refund      │ │
│  └──────────────────────────┬─────────────────────────────┘ │
└─────────────────────────────┼───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Repository Layer (Trait)                   │
│  ┌───────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐   │
│  │ShowRepo   │ │SeatRepo    │ │BookingRepo │ │PaymentRepo│  │
│  └───────────┘ └────────────┘ └────────────┘ └──────────┘   │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Persistence Layer                          │
│  ┌─────────────────────────┐  ┌─────────────────────────┐   │
│  │ In-MemoryRepo (default) │  │ PostgresRepo (future)   │   │
│  └─────────────────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Domain Models

### 4.1 Enumerations

#### `SeatStatus`
```
Available  — seat is free and can be locked
Locked     — seat is temporarily held (by a user, with TTL)
Booked     — seat is permanently confirmed after payment
```

#### `BookingStatus`
```
Pending           — booking created, payment not started
PaymentPending    — payment initiated, awaiting gateway
PaymentSuccess    — payment confirmed, seats being booked
PaymentFailed     — payment failed, seats released
Success           — booking complete, seats confirmed
Cancelled         — user cancelled before payment
Expired           — booking timed out before payment
```

#### `PaymentStatus`
```
Pending   — payment initiated, awaiting gateway response
Success   — payment confirmed by gateway
Failed    — payment declined / error
Refunded  — payment reversed
```

#### `LockStatus` (internal)
```
Active          — lock is held and valid
Expired         — lock TTL reached, seat should be released
Released        — lock was manually released (user cancelled)
```

### 4.2 Struct Definitions

#### `User`
| Field | Type | Constraints |
|-------|------|-------------|
| `user_id` | `String` (UUID) | Not empty, immutable after creation |
| `user_name` | `String` | 1–100 chars, non-empty |
| `email` | `String` | Valid email format |
| `created_at` | `DateTime<Utc>` | Auto-set on creation |

#### `Show`
| Field | Type | Constraints |
|-------|------|-------------|
| `show_id` | `String` (UUID) | Primary key |
| `show_name` | `String` | Non-empty, max 200 chars |
| `theatre_name` | `String` | Non-empty, max 100 chars |
| `screen_number` | `u32` | >= 1 |
| `start_time` | `DateTime<Utc>` | Must be in the future |
| `end_time` | `DateTime<Utc>` | Must be > start_time |
| `price_per_seat` | `Decimal` (f64) | > 0 |
| `total_seats` | `u32` | > 0 |
| `created_at` | `DateTime<Utc>` | Auto-set |

#### `Seat`
| Field | Type | Constraints |
|-------|------|-------------|
| `seat_id` | `String` (UUID) | Primary key |
| `seat_number` | `String` | e.g., "A1", "B12"; non-empty |
| `row_label` | `String` | e.g., "A", "B" |
| `seat_type` | `SeatType` enum | Standard, Premium, Recliner |
| `price_modifier` | `f64` | Multiplier on show price (e.g., 1.5 for Premium) |
| `show_id` | `String` (UUID) | FK to Show |
| `status` | `SeatStatus` | Current status |
| `booked_by` | `Option<User>` | Set when Booked |
| `locked_by` | `Option<User>` | Set when Locked |
| `locked_at` | `Option<DateTime<Utc>>` | When lock was acquired |
| `lock_expires_at` | `Option<DateTime<Utc>>` | When lock expires (TTL boundary) |
| `lock_id` | `Option<String>` (UUID) | Unique per lock session |

#### `SeatLock` (standalone entity, not embedded in Seat)
| Field | Type | Constraints |
|-------|------|-------------|
| `lock_id` | `String` (UUID) | Primary key |
| `user_id` | `String` (UUID) | Who holds the lock |
| `show_id` | `String` (UUID) | Show this lock belongs to |
| `seat_ids` | `Vec<String>` | Seats locked in this session |
| `status` | `LockStatus` | Active / Expired / Released |
| `created_at` | `DateTime<Utc>` | Lock acquisition time |
| `expires_at` | `DateTime<Utc>` | Lock expiry time |
| `extended_count` | `u32` | Number of times lock was extended (max 2) |

#### `Booking`
| Field | Type | Constraints |
|-------|------|-------------|
| `booking_id` | `String` (UUID) | Primary key |
| `user_id` | `String` (UUID) | Booking user |
| `show_id` | `String` (UUID) | Target show |
| `seat_ids` | `Vec<String>` | All seats in this booking |
| `status` | `BookingStatus` | Current status |
| `payment_id` | `Option<String>` | Set when payment initiated |
| `total_amount` | `f64` | Computed from seat prices |
| `lock_id` | `Option<String>` | Link to the SeatLock that reserved seats |
| `created_at` | `DateTime<Utc>` | Booking creation time |
| `expires_at` | `DateTime<Utc>` | Booking / lock expiry time |
| `confirmed_at` | `Option<DateTime<Utc>>` | Set on confirmation |
| `cancelled_at` | `Option<DateTime<Utc>>` | Set on cancellation |

#### `Payment`
| Field | Type | Constraints |
|-------|------|-------------|
| `payment_id` | `String` (UUID) | Primary key |
| `payment_intent_id` | `String` (UUID) | External reference (gateway) |
| `booking_id` | `String` (UUID) | FK to Booking |
| `user_id` | `String` (UUID) | Paying user |
| `amount` | `f64` | Must match booking total |
| `currency` | `String` | "INR" (hardcoded) |
| `status` | `PaymentStatus` | Current status |
| `gateway_name` | `String` | e.g., "mock", "razorpay" |
| `gateway_response` | `Option<String>` | Raw gateway response |
| `created_at` | `DateTime<Utc>` | Payment initiation time |
| `confirmed_at` | `Option<DateTime<Utc>>` | Gateway confirmation time |
| `failed_at` | `Option<DateTime<Utc>>` | Failure timestamp |
| `refunded_at` | `Option<DateTime<Utc>>` | Refund timestamp |

#### `QueueEntry` (internal)
| Field | Type | Constraints |
|-------|------|-------------|
| `queue_id` | `String` (UUID) | Primary key |
| `user_id` | `String` (UUID) | Requesting user |
| `show_id` | `String` (UUID) | Show for which user wants seats |
| `requested_seat_ids` | `Vec<String>` | Seats user wants to book |
| `status` | `QueueStatus` | Waiting / Processing / Completed / Expired |
| `position` | `u32` | Position in queue |
| `created_at` | `DateTime<Utc>` | When user entered queue |
| `processed_at` | `Option<DateTime<Utc>>` | When lock was granted / denied |

#### `SeatType` (enum)
```
Standard  — Regular seat, price_modifier = 1.0
Premium   — Premium seat, price_modifier = 1.5
Recliner  — Recliner seat, price_modifier = 2.0
```

---

## 5. Core Feature: Seat Inventory Management

### 5.1 Show CRUD

| Operation | Description |
|-----------|-------------|
| Create Show | Admin creates a show with name, times, seat layout |
| Auto-generate Seats | On show creation, system generates all Seat records from layout config |
| List Shows | Returns all shows with availability summary |
| Get Show | Returns show detail + seat map (status per seat) |
| Get Seat Layout | Returns 2D grid of seats with current status |

### 5.2 Seat Layout Generation

On `Create Show`, a seat layout is auto-generated:

- Input: `rows: ["A","B","C","D"], seats_per_row: [10, 10, 10, 10]`
- For each row, for each seat number in that row:
  - Generate `seat_id = UUID`
  - Generate `seat_number = "{row}{seat_num}"` e.g., "A1", "A2"
  - Assign `SeatType` based on row: first row = Premium, last row = Recliner, rest = Standard
  - `status = Available`

### 5.3 Seat Availability Query

- `GET /shows/{show_id}/seats` returns all seats for a show
- Each seat in response includes current `status`, and if `Locked`, the `lock_expires_at` timestamp
- Response is paginated: `?page=1&limit=100`

---

## 6. Core Feature: Seat Locking System

### 6.1 Overview

The seat locking system is the **core competitive-diff** mechanism. When a user selects seats, those seats must be atomically locked (held) for a fixed duration while the user proceeds to payment. No other user can select or book locked seats.

### 6.2 Lock Acquisition Flow

```
User selects seats → POST /bookings/lock
                                    │
          ┌─────────────────────────┴─────────────────────────┐
          │                                                   │
          ▼                                                   ▼
  ┌───────────────┐                              ┌──────────────────┐
  │ Acquire per-show│                             │   Queue if      │
  │ Mutex lock     │                              │   concurrent>N  │
  └───────┬───────┘                              └──────────────────┘
          │
          ▼
  ┌──────────────────────────────────────────────────────────────┐
  │ 1. Validate all seats belong to the requested show_id        │
  │ 2. Validate all seats are Available (not Locked, not Booked) │
  │ 3. Check seat lock TTL hasn't expired (background cleanup)   │
  │ 4. Generate lock_id (UUID)                                   │
  │ 5. Set seat.status = Locked for all seats                    │
  │ 6. Set seat.locked_by = current_user                         │
  │ 7. Set seat.locked_at = now                                  │
  │ 8. Set seat.lock_expires_at = now + LOCK_TTL (default 5min)  │
  │ 9. Create SeatLock record with status = Active               │
  │ 10. Create Booking record with status = Pending              │
  │ 11. Return lock_id, booking_id, expires_at                   │
  └──────────────────────────────────────────────────────────────┘
          │
          ▼
  ┌──────────────────────────────────────┐
  │ Release per-show Mutex               │
  └──────────────────────────────────────┘
```

### 6.3 Lock Configuration

| Parameter | Default | Config Key | Description |
|-----------|---------|------------|-------------|
| `LOCK_TTL_SECONDS` | 300 (5 min) | `SEAT_LOCK_TTL_SECS` | How long a lock is held |
| `LOCK_MAX_EXTENSIONS` | 2 | `SEAT_LOCK_MAX_EXTENSIONS` | Max times a user can extend |
| `LOCK_EXTENSION_DURATION` | 120 (2 min) | `SEAT_LOCK_EXTENSION_SECS` | Extra time per extension |
| `LOCK_GRACE_PERIOD_SECS` | 30 | `SEAT_LOCK_GRACE_PERIOD_SECS` | Buffer before lock truly expires |

### 6.4 Lock Extension

- User can call `POST /bookings/{booking_id}/extend-lock`
- Each extension adds `LOCK_EXTENSION_DURATION` to `lock_expires_at`
- Maximum `LOCK_MAX_EXTENSIONS` total extensions per lock session
- If `extended_count >= LOCK_MAX_EXTENSIONS`, return `409 Conflict` with message "Maximum lock extensions reached"
- Extension only works if:
  - Booking status is `Pending` or `PaymentPending`
  - Lock has not already expired
  - User owns the lock

### 6.5 Lock Release (Manual)

- User calls `DELETE /bookings/{booking_id}/lock`
- All seats in the lock session are set back to `Available`
- `SeatLock.status = Released`
- `Booking.status = Cancelled`
- No refund is issued (payment hasn't been made)

### 6.6 Lock Expiration (Automatic)

- A background async task runs every **10 seconds**
- Task queries all `SeatLock` records where:
  - `status = Active`
  - `expires_at < now - GRACE_PERIOD`
- For each expired lock:
  - Set all linked seats to `Available`
  - Set `SeatLock.status = Expired`
  - Set `Booking.status = Expired` (if booking still exists and is Pending)
  - Emit an internal event (for potential notification)

### 6.7 Concurrency Guarantee

- **Per-show Mutex**: Before any lock acquisition, acquire a `RwLock<Mux>` scoped to the `show_id`
- Only **one lock acquisition** per show can proceed at a time
- Within the critical section, a second check (not just DB state but in-memory snapshot) validates seat availability
- If any seat is already `Locked` or `Booked`, the entire request is rejected atomically

### 6.8 Edge Cases

| Scenario | Behavior |
|----------|----------|
| User tries to lock already-locked seat | `409 Conflict` — "Seat {X} is currently held by another user" |
| User tries to lock already-booked seat | `409 Conflict` — "Seat {X} is already booked" |
| User tries to lock seats from different shows in one request | `400 Bad Request` — "All seats must belong to the same show" |
| User tries to lock 0 seats | `400 Bad Request` — "At least one seat must be selected" |
| User tries to lock more than 10 seats | `400 Bad Request` — "Maximum 10 seats per booking" |
| Lock expires while user is mid-payment | Payment callback receives expired lock_id → `PaymentSuccess` still calls `confirm_booking`, which must verify seats are still `Locked` by this user before confirming |
| Double-lock attempt by same user on same seats | `409 Conflict` — "Seats already locked by you" |

---

## 7. Core Feature: Request Queue & Concurrency Control

### 7.1 Problem Statement

In a high-traffic scenario (e.g., popular movie premiere ticket sales), many users may attempt to select the same seats simultaneously. Without a queue, all would receive the seat at once (race condition). The queue ensures **fair, ordered access**.

### 7.2 Queue Design

The queue is **token-based with per-show ordering**. It is NOT a first-come-first-served global queue — instead, each show has its own queue. This prevents a user browsing Show A from being blocked by users booking Show B.

### 7.3 Queue Flow

```
User A: POST /shows/{show_id}/queue/join  (wants seats: A1, A2)
User B: POST /shows/{show_id}/queue/join  (wants seats: A3, A4)
User C: POST /shows/{show_id}/queue/join  (wants seats: A1, A5)

Queue for show_id = [UserA, UserB, UserC]
Position:           [1,     2,     3    ]

Processing thread picks UserA first:
  → Tries to lock A1, A2 → SUCCESS
  → Removes from queue, advances

Processing thread picks UserB:
  → Tries to lock A3, A4 → SUCCESS
  → Removes from queue, advances

Processing thread picks UserC:
  → Tries to lock A1 (locked by A!), A5 → FAIL
  → UserC gets 409 with list of unavailable seats
  → UserC must re-select seats (back to seat map)
```

### 7.4 Queue Entry Lifecycle

| State | Description |
|-------|-------------|
| `Waiting` | User is in queue, awaiting processing |
| `Processing` | User's lock attempt is being processed (max 10 seconds) |
| `Locked` | Lock was successfully acquired — user proceeds to payment |
| `Conflict` | One or more requested seats were unavailable — user must re-select |
| `Expired` | User's processing window expired without a response |

### 7.5 Queue Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `QUEUE_PROCESSING_TIMEOUT_SECS` | 10 | Time allowed for a queue entry before being marked expired |
| `QUEUE_MAX_CONCURRENT_PER_SHOW` | 3 | Max simultaneous lock attempts per show |
| `QUEUE_POLL_INTERVAL_MS` | 500 | How often the queue processor checks for new entries |

### 7.6 Queue Processing Logic

- A dedicated async background task (`QueueProcessor`) polls the queue per show
- At most `QUEUE_MAX_CONCURRENT_PER_SHOW` entries are in `Processing` state per show
- Processing is non-blocking: the HTTP request for `/queue/join` returns immediately with `queue_id` and `position`
- User polls `GET /queue/{queue_id}/status` to check their position and result

### 7.7 Seat Availability Check (during queue processing)

```
FOR each requested seat:
    IF seat.status == Available:
        continue
    ELSE IF seat.status == Locked AND seat.lock_expires_at < now:
        // Stale lock — treat as available (cleanup runs in parallel)
        continue
    ELSE:
        // Seat is not available
        add to conflict_seats list
        continue

IF conflict_seats is empty:
    proceed with lock acquisition (same as Section 6.2)
ELSE:
    mark queue entry as Conflict with conflict_seats list
```

---

## 8. Core Feature: Booking Flow

### 8.1 Complete Booking State Machine

```
                                   [lock acquired]
                                          │
                                          ▼
                                    ┌──────────┐
                                    │  Pending │
                                    └────┬─────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
          [user cancels]          [user pays]           [TTL expires]
                    │                    │                    │
                    ▼                    ▼                    ▼
            ┌────────────┐       ┌────────────────┐    ┌──────────┐
            │ Cancelled  │       │PaymentPending  │    │  Expired │
            └────────────┘       └───────┬────────┘    └──────────┘
                                         │
                          ┌──────────────┴──────────────┐
                          │                             │
               [payment success]           [payment failed]
                          │                             │
                          ▼                             ▼
                   ┌──────────┐                 ┌────────────┐
                   │ Success  │                 │PaymentFailed│
                   └──────────┘                 └────────────┘
```

### 8.2 Booking Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/shows/{show_id}/seats/lock` | Lock seats and create booking |
| `POST` | `/bookings/{booking_id}/extend-lock` | Extend lock TTL |
| `DELETE` | `/bookings/{booking_id}/lock` | Release lock manually |
| `GET` | `/bookings/{booking_id}` | Get booking status and details |
| `POST` | `/bookings/{booking_id}/cancel` | Cancel booking (user-initiated) |
| `GET` | `/bookings/user/{user_id}` | List all bookings for a user |
| `GET` | `/shows/{show_id}/seats` | Get seat layout with availability |

### 8.3 Seat Lock + Booking Creation

**Request:**
```json
POST /shows/{show_id}/seats/lock
Authorization: User-Id: {user_id}
{
  "seat_ids": ["uuid-1", "uuid-2", "uuid-3"]
}
```

**Response (201 Created):**
```json
{
  "booking_id": "uuid-booking",
  "lock_id": "uuid-lock",
  "show_id": "uuid-show",
  "seats": [
    { "seat_id": "uuid-1", "seat_number": "A1", "status": "Locked" },
    { "seat_id": "uuid-2", "seat_number": "A2", "status": "Locked" },
    { "seat_id": "uuid-3", "seat_number": "A3", "status": "Locked" }
  ],
  "total_amount": 450.00,
  "expires_at": "2026-04-28T14:05:00Z",
  "status": "Pending"
}
```

**Error Responses:**
| Code | Condition |
|------|-----------|
| `400` | Seats not in show / empty / > 10 seats |
| `404` | Show not found |
| `409` | One or more seats unavailable |
| `429` | Too many requests from this user |

### 8.4 Booking Confirmation (Post-Payment)

After `PaymentService.payment_callback()` receives success:
1. Verify `Booking.status == PaymentPending`
2. Verify `lock_expires_at >= now` (or within grace period)
3. For each seat in `Booking.seats`:
   - Verify `seat.status == Locked && seat.locked_by.user_id == booking.user_id`
   - Set `seat.status = Booked`
   - Set `seat.booked_by = booking.user`
   - Clear `locked_by`, `locked_at`, `lock_expires_at`, `lock_id`
4. Set `Booking.status = Success`
5. Set `Booking.confirmed_at = now`
6. Set `SeatLock.status = Expired` (lock is now consumed)
7. Return confirmed booking

If any seat fails the verification (was taken by another booking or expired lock):
- Log the inconsistency
- Set `Booking.status = SuccessPartial` with a list of confirmed seats and failed seats
- Create a `CompensationLog` entry
- Trigger refund for the failed seats' portion

### 8.5 Booking Cancellation

- `DELETE /bookings/{booking_id}/lock` or `POST /bookings/{booking_id}/cancel`
- Only allowed if `Booking.status ∈ {Pending, PaymentPending}`
- Seats released: `seat.status = Available`, lock fields cleared
- `SeatLock.status = Released`
- `Booking.status = Cancelled`

### 8.6 Booking Expiration

- Background task (every 10s) scans `Booking` where `status = Pending` and `expires_at < now - GRACE_PERIOD`
- Calls the same release logic as cancellation

---

## 9. Core Feature: Payment Processing

### 9.1 Payment Flow

```
Booking Created (Pending)
       │
       ▼
POST /bookings/{booking_id}/payment/initiate
       │
       ▼
┌─────────────────────────────────────────┐
│ PaymentService.initiate_payment()       │
│ - Create Payment record (Pending)       │
│ - Update Booking.status = PaymentPending│
│ - Call mock gateway with payment_intent │
└─────────────────┬───────────────────────┘
                  │ (async, simulated 2-5s delay)
                  ▼
┌─────────────────────────────────────────┐
│ Gateway "callback" → payment_callback()│
│                                         │
│ SUCCESS:                                 │
│   - Payment.status = Success            │
│   - BookingService.confirm_booking()     │
│                                         │
│ FAILED:                                  │
│   - Payment.status = Failed              │
│   - BookingService.cancel_booking()      │
│   - Seats released                       │
└─────────────────────────────────────────┘
```

### 9.2 Payment Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/bookings/{booking_id}/payment/initiate` | Start payment |
| `POST` | `/payments/{payment_id}/callback` | Gateway callback (simulated) |
| `GET` | `/payments/{payment_id}` | Get payment status |
| `POST` | `/payments/{payment_id}/refund` | Admin refund |

### 9.3 Mock Payment Gateway

Since no real gateway is integrated, a **mock gateway** simulates real-world behavior:

- `POST /mock-gateway/pay` accepts `{ payment_intent_id, amount, card_last4, simulate_failure: bool }`
- Randomly succeeds (80%) or fails (20%) by default
- Accepts `simulate_failure: true` to force failure
- Accepts `simulate_delay_ms` to artificially delay response
- Returns `{ status: "SUCCESS" | "FAILED", gateway_reference: "GW-{uuid}" }`

### 9.4 Payment Idempotency

- `initiate_payment()` must be idempotent
- If called twice with same `booking_id` and `payment_id` already exists, return existing payment
- Duplicate `payment_callback` calls with same `payment_intent_id` are ignored (already processed)

### 9.5 Payment Timeout

- If payment is not confirmed within `PAYMENT_TIMEOUT_SECS = 600` (10 min) from `initiate_payment()`:
  - `Payment.status` stays `Pending`
  - `Booking.status = Expired`
  - Seats released
- A background task checks for stale payments every 30 seconds

---

## 10. Core Feature: HTTP API Layer

### 10.1 Technology Stack

- **Framework**: Axum 0.7+
- **Runtime**: Tokio
- **Serialization**: Serde + JSON
- **Validation**: validator crate

### 10.2 API Endpoints (Complete)

#### Shows
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/admin/shows` | Admin | Create a new show |
| `GET` | `/shows` | Public | List all shows |
| `GET` | `/shows/{show_id}` | Public | Get show details |
| `GET` | `/shows/{show_id}/seats` | Public | Get seat layout |
| `DELETE` | `/admin/shows/{show_id}` | Admin | Cancel show (refund all bookings) |

#### Booking & Locking
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/shows/{show_id}/seats/lock` | User-Id header | Lock seats |
| `POST` | `/bookings/{booking_id}/extend-lock` | User-Id header | Extend lock TTL |
| `DELETE` | `/bookings/{booking_id}/lock` | User-Id header | Release lock |
| `GET` | `/bookings/{booking_id}` | User-Id header | Get booking |
| `POST` | `/bookings/{booking_id}/cancel` | User-Id header | Cancel booking |
| `GET` | `/bookings/user/{user_id}` | User-Id header | User's bookings |

#### Payment
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/bookings/{booking_id}/payment/initiate` | User-Id header | Start payment |
| `GET` | `/payments/{payment_id}` | User-Id header | Get payment status |
| `POST` | `/payments/{payment_id}/callback` | Internal | Gateway callback |
| `POST` | `/payments/{payment_id}/refund` | Admin | Issue refund |

#### Queue (optional — can be simplified out)
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/shows/{show_id}/queue/join` | User-Id header | Join show queue |
| `GET` | `/queue/{queue_id}/status` | User-Id header | Poll queue status |
| `DELETE` | `/queue/{queue_id}` | User-Id header | Leave queue |

### 10.3 Request / Response Formats

#### Standard Success Response
```json
{
  "success": true,
  "data": { ... },
  "timestamp": "2026-04-28T12:00:00Z"
}
```

#### Standard Error Response
```json
{
  "success": false,
  "error": {
    "code": "SEATS_UNAVAILABLE",
    "message": "One or more selected seats are not available",
    "details": {
      "unavailable_seats": ["uuid-1", "uuid-3"]
    }
  },
  "timestamp": "2026-04-28T12:00:00Z"
}
```

### 10.4 Authentication

- No JWT / OAuth for MVP
- User identification via `X-User-Id` header (UUID)
- Admin operations via `X-Admin-Token` header (static secret from env)
- Future: JWT-based auth with role claims

### 10.5 Rate Limiting

| Endpoint | Limit |
|----------|-------|
| `POST /shows/{show_id}/seats/lock` | 5 requests/min/user |
| `POST /bookings/{booking_id}/payment/initiate` | 3 requests/min/user |
| All other endpoints | 60 requests/min/user |

---

## 11. Core Feature: Admin / Show Management

### 11.1 Admin Capabilities

| Action | Description |
|--------|-------------|
| Create Show | Name, times, seat layout config |
| Cancel Show | All associated bookings refunded, seats released |
| View All Bookings | Across all shows |
| View Show Analytics | Occupancy rate, revenue per show |
| Manual Seat Override | Force-release a locked seat (with audit log) |

### 11.2 Seat Layout Configuration (on show creation)

```json
POST /admin/shows
{
  "show_name": "Avengers: Endgame",
  "theatre_name": "PVR Nexus",
  "screen_number": 1,
  "start_time": "2026-05-01T14:00:00Z",
  "end_time": "2026-05-01T17:00:00Z",
  "price_per_seat": 250.00,
  "seat_layout": {
    "rows": [
      { "row": "A", "seats": 10, "type": "Premium" },
      { "row": "B", "seats": 10, "type": "Standard" },
      { "row": "C", "seats": 10, "type": "Standard" },
      { "row": "D", "seats": 10, "type": "Recliner" }
    ]
  }
}
```

---

## 12. Concurrency & Thread Safety

### 12.1 Shared State Management

All shared mutable state is protected by appropriate Rust synchronization primitives:

| Resource | Primitive | Rationale |
|----------|-----------|-----------|
| Seat data | `RwLock<HashMap<String, Seat>>` | Read-heavy (availability queries), write-once (booking) |
| Per-show lock queue | `Mutex<Vec<QueueEntry>>` | Single-threaded processing per show |
| Booking data | `RwLock<HashMap<String, Booking>>` | Read-heavy, occasional writes |
| Payment data | `RwLock<HashMap<String, Payment>>` | Write-once, then read |
| Global config | `OnceLock<AppConfig>` | Initialized once at startup |

### 12.2 Per-Show Critical Section

```rust
// Acquiring per-show mutex before lock operations
let show_mutex = self.show_locks.get_or_init(&show_id);
let _guard = show_mutex.lock().await;

// Within this guard:
// 1. Re-check seat availability
// 2. Acquire all seat locks atomically
// 3. Persist to repository
// 4. Release guard
```

### 12.3 Double-Checked Locking Pattern

The per-show mutex prevents concurrent lock requests from racing, but we ALSO double-check within the critical section:

```rust
async fn lock_seats(&self, show_id: &str, seat_ids: &[String], user_id: &str) -> Result<LockResult> {
    let show_guard = self.show_locks.get(&show_id).unwrap().lock().await;

    // CRITICAL SECTION START
    let seats = self.seat_repo.get_seats(seat_ids).await;

    // Validate all seats are available
    for seat in &seats {
        if seat.status != SeatStatus::Available {
            return Err(AppError::SeatUnavailable(seat.seat_id.clone()));
        }
    }

    // All good — proceed with lock
    // ... (atomic lock acquisition)
    // CRITICAL SECTION END

    drop(show_guard);
    // Outside critical section — no more seat mutations possible for this show
    // ... (create booking, etc.)
}
```

### 12.4 Async/Await Architecture

- All service methods are `async fn`
- All repository methods are `async fn`
- Tokio runtime with multi-threaded scheduler
- Tokio `Interval` for background tasks (lock expiration, queue polling, payment timeout)
- No `spawn_blocking` needed for in-memory repos (CPU-light operations)

---

## 13. Error Handling Strategy

### 13.1 Error Types

```rust
pub enum AppError {
    // 400 Bad Request
    ValidationError(String),

    // 404 Not Found
    ShowNotFound(String),
    BookingNotFound(String),
    SeatNotFound(String),
    PaymentNotFound(String),
    UserNotFound(String),

    // 409 Conflict
    SeatUnavailable(String),          // One seat unavailable
    SeatsUnavailable(Vec<String>),    // Multiple seats
    BookingAlreadyProcessed(String),  // Booking not in processable state
    LockNotOwnedByUser(String),       // User tried to unlock another's lock
    LockMaxExtensionsReached,

    // 410 Gone
    LockExpired(String),
    BookingExpired(String),

    // 422 Unprocessable Entity
    PaymentMismatch(String),          // Payment amount != booking amount

    // 429 Too Many Requests
    RateLimitExceeded,

    // 500 Internal Server Error
    InternalError(String),
    RepositoryError(String),
}
```

### 13.2 Error Propagation

- Service layer returns `Result<T, AppError>` (no panics for business logic)
- Panics are **reserved only for** unrecoverable programmer errors (e.g., `unwrap()` on known-non-None value in dev)
- All repository methods return `Result<T, AppError>`
- HTTP layer converts `AppError` to appropriate HTTP response via Axum `From<AppError> for StatusCode`

### 13.3 Logging

- All errors logged with: timestamp, error type, user_id, show_id, booking_id (if applicable), stack trace
- Log levels: `ERROR` for business errors, `WARN` for expected conflicts, `INFO` for successful operations
- No PII (email, card numbers) in logs

---

## 14. Data Persistence Layer

### 14.1 Repository Pattern

```
Repository Trait (interface)
    │
    ├── InMemoryRepository  (default, for dev / tests)
    │
    └── PostgresRepository  (future, for production)
```

All repository traits use async methods:

```rust
pub trait SeatRepository: Send + Sync {
    async fn save(&self, seat: Seat) -> Result<Seat, AppError>;
    async fn find_by_id(&self, seat_id: &str) -> Result<Option<Seat>, AppError>;
    async fn find_by_show(&self, show_id: &str) -> Result<Vec<Seat>, AppError>;
    async fn find_by_ids(&self, seat_ids: &[String]) -> Result<Vec<Seat>, AppError>;
    async fn find_by_show_and_status(
        &self,
        show_id: &str,
        status: SeatStatus,
    ) -> Result<Vec<Seat>, AppError>;
    async fn update_status(&self, seat_id: &str, status: SeatStatus) -> Result<Seat, AppError>;
}
```

### 14.2 Transaction Support

For the Postgres repository (future), lock acquisition MUST be atomic:

```sql
BEGIN;
-- Per-show lock acquired (SELECT FOR UPDATE on show row)

SELECT * FROM seats
WHERE seat_id = ANY($1) AND show_id = $2 AND status = 'Available'
FOR UPDATE;  -- Row-level lock

-- If count == len(seat_ids):
UPDATE seats SET status = 'Locked', locked_by = $3, lock_expires_at = $4
WHERE seat_id = ANY($1);

INSERT INTO bookings (...);
INSERT INTO seat_locks (...);

COMMIT;
```

---

## 15. Configuration & Environment

### 15.1 Config File: `config.toml`

```toml
[app]
host = "0.0.0.0"
port = 8080
log_level = "info"

[seat_lock]
ttl_seconds = 300
max_extensions = 2
extension_seconds = 120
grace_period_seconds = 30

[queue]
processing_timeout_seconds = 10
max_concurrent_per_show = 3
poll_interval_ms = 500

[payment]
timeout_seconds = 600
mock_gateway_delay_ms = 2000
mock_gateway_failure_rate = 0.2

[rate_limit]
lock_requests_per_min = 5
payment_requests_per_min = 3
default_requests_per_min = 60

[persistence]
driver = "in_memory"  # or "postgres"
```

### 15.2 Environment Variable Overrides

| Env Var | Overrides Config Key | Example |
|---------|---------------------|---------|
| `APP_PORT` | `app.port` | `8080` |
| `LOG_LEVEL` | `app.log_level` | `debug` |
| `SEAT_LOCK_TTL_SECS` | `seat_lock.ttl_seconds` | `180` |
| `DATABASE_URL` | (connection string) | `postgres://...` |

---

## 16. Non-Functional Requirements

### 16.1 Performance

| Metric | Target |
|--------|--------|
| Lock acquisition (single seat) | < 50ms p99 |
| Lock acquisition (10 seats) | < 200ms p99 |
| Seat availability query | < 20ms p99 |
| Booking confirmation | < 100ms p99 |
| Payment callback processing | < 500ms p99 |
| Concurrent lock attempts (per show) | Support 100+ simultaneous requests without data races |

### 16.2 Reliability

- No data loss on lock expiration (background task runs reliably)
- Payment callback is idempotent (duplicate calls are safe)
- All state changes are persisted before returning HTTP 201

### 16.3 Observability

- Structured JSON logging (tracing crate)
- Request ID propagated through all layers
- Health check endpoint: `GET /health` returns `{ "status": "ok", "uptime_seconds": N }`
- Metrics (future): request latency histograms, lock contention counters

### 16.4 Testing Requirements

| Test Type | Coverage Target | Description |
|-----------|----------------|-------------|
| Unit Tests | Core services | Seat locking logic, price calculation, state transitions |
| Integration Tests | Repository layer | In-memory repo correctness |
| Concurrency Tests | Lock acquisition | 50 concurrent requests for same seats — exactly 1 succeeds |
| API Tests | HTTP layer | All endpoints with valid and invalid inputs |

---

## 17. Phased Implementation Roadmap

### Phase 1: Foundation & Core Locking (Weeks 1–2)
- [ ] Migrate to async Rust (Tokio, Axum)
- [ ] Implement `AppConfig` loading from `config.toml` + env vars
- [ ] Rewrite domain models with Serde derive
- [ ] Implement `AppError` enum with full error types
- [ ] Async repository traits + in-memory implementation
- [ ] **Core seat locking service** with per-show Mutex
- [ ] **Background lock expiration task**
- [ ] Basic HTTP endpoints: lock, unlock, get booking
- [ ] Unit tests for lock acquisition logic

### Phase 2: Booking Flow & Payment (Weeks 3–4)
- [ ] Full booking state machine implementation
- [ ] Payment service with mock gateway
- [ ] Payment callback integration
- [ ] Booking confirmation with seat promotion
- [ ] Booking cancellation and seat release
- [ ] `POST /bookings/{booking_id}/extend-lock`
- [ ] Concurrency test: 50 concurrent requests for same seats
- [ ] API integration tests

### Phase 3: Show Management & Admin (Week 5)
- [ ] Show CRUD endpoints
- [ ] Seat layout auto-generation on show creation
- [ ] Seat availability query endpoint
- [ ] Admin endpoints (create show, cancel show)
- [ ] User booking history endpoint

### Phase 4: Queue System (Week 6)
- [ ] Queue entry creation and management
- [ ] Queue processor background task
- [ ] Queue status polling endpoint
- [ ] Conflict detection and reporting
- [ ] Queue integration with locking service

### Phase 5: Polish & Production Readiness (Week 7)
- [ ] Structured logging with tracing
- [ ] Health check endpoint
- [ ] Rate limiting middleware
- [ ] Postgres repository (replace in-memory)
- [ ] Docker / Docker Compose setup
- [ ] Full integration test suite
- [ ] Performance benchmark suite

---

## 18. Out of Scope

| Item | Reason |
|------|--------|
| Frontend (React, mobile, etc.) | Backend-only delivery |
| Real payment gateway (Razorpay, Stripe) | Integration complexity; mocked for MVP |
| User authentication / registration | Out of scope; user_id passed as header |
| JWT / OAuth | Future phase |
| Distributed / multi-node deployment | Single-node MVP |
| Email / SMS notifications | Future phase |
| Refunds (manual / auto) | Payment gateway concern |
| Seat selection UI drag-and-drop | Frontend concern |
| Multi-theatre / multi-city | Single instance |
| Caching layer (Redis) | In-memory sufficient for MVP |
| API versioning | Unnecessary for MVP |

---

## 19. Glossary

| Term | Definition |
|------|-----------|
| **Lock** | Temporary hold on a seat, preventing others from booking it, with a time-to-live (TTL) |
| **Lock TTL** | Duration a seat lock remains valid before automatic expiration |
| **Lock Extension** | User-initiated prolongation of an active lock |
| **Queue** | Per-show ordered waiting list for users attempting to lock seats during high concurrency |
| **Lock Acquisition** | Atomic operation of marking one or more seats as `Locked` for a specific user |
| **Payment Intent** | External payment reference ID (UUID), generated before gateway call |
| **Seat Type** | Category of seat (Standard, Premium, Recliner) with a price modifier |
| **Grace Period** | Buffer window after lock expiry during which the lock is considered "soft expired" (not yet released) |
| **Per-Show Mutex** | Synchronization primitive ensuring only one lock operation proceeds at a time for a given show |
| **Double-Checked Locking** | Pattern where availability is checked both before and within the critical section |
| **Booking Confirmation** | Transition of seats from `Locked` to `Booked` after successful payment |
| **Seat Release** | Transition of seats from `Locked` back to `Available` (on cancel, expiry, or payment failure) |

---

## 20. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-28 | Claude | Initial draft from existing codebase analysis |
