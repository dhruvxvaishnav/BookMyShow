# Cineplex - Full-Stack Movie Ticket Booking Platform

Cineplex is a full-stack cinema booking platform inspired by BookMyShow. It supports movie discovery, venue and showtime selection, interactive seat booking, timed seat locks, payment flow, booking confirmation, user booking history, and admin show management.

This project is designed to demonstrate real-world product engineering, backend system design, Rust service architecture, and a polished Next.js user experience.

> Portfolio note: screenshots and demo videos can be added later in the sections marked below.

## Project Snapshot

| Area | Details |
| --- | --- |
| Frontend | Next.js App Router, React, TypeScript, CSS Modules |
| Backend | Rust, Axum, Tokio, SQLx, layered crate architecture |
| Database | PostgreSQL |
| Cache / infra | Redis-ready Docker setup |
| Auth | JWT user auth and admin auth |
| Payments | Mock card flow plus Stripe integration hooks |
| Deployment | Docker Compose for frontend, backend, Postgres, Redis |

## Why This Project Matters

Booking platforms are deceptively complex. The hard problem is not just showing movies; it is preventing double booking when multiple users compete for the same seats, expiring abandoned locks, handling payment retries, and keeping booking state consistent.

This project models those problems directly:

- Seats can be locked for a short period before payment.
- Expired locks are released by background workers.
- Booking state moves through a clear lifecycle.
- Payment initiation is retry-safe.
- Admin-created shows are linked to movies and venues so they appear in user flows.
- User-facing and admin-facing workflows share the same backend domain model.

## Demo Placeholders

Add screenshots here when ready:

- Home / movie discovery screen
- Movie detail and showtime selection screen
- Seat selection screen with Normal, Luxe, and IMAX layouts
- Payment screen
- Booking confirmation ticket
- My Bookings screen
- Admin dashboard and create-show flow

Add demo video here:

- End-to-end booking walkthrough
- Admin creates a show, user books it

## Core Features

### User Experience

- Browse movies with poster images, ratings, language, genre, and duration.
- View movie details and available showtimes.
- Filter showtimes by date, city, and format.
- Choose between show formats such as Normal, Luxe, and IMAX.
- Select seats from an interactive seat map.
- Choose ticket quantity from 1 to 10 and auto-select adjacent seats.
- Lock selected seats before payment.
- Complete payment through a mock card payment flow.
- View a polished booking confirmation ticket.
- View upcoming and past bookings correctly based on show time.

### Admin Experience

- Admin login and protected admin pages.
- Dashboard with revenue, active bookings, locked seats, and show status.
- Create new shows with custom seat layout.
- Select or auto-create a movie from the show name.
- Link shows to venues so they appear correctly on user pages.
- View show analytics such as occupancy and revenue.
- Inspect bookings and audit logs.
- Cancel shows and release associated booking state.

### Booking and Payment Lifecycle

- Seat locking with TTL.
- Booking statuses including pending, payment pending, success, partial success, failed, expired, and cancelled.
- Payment initiation and mock gateway completion.
- Expired payment and lock cleanup.
- Compensation log support for partial booking failures.

## System Design Highlights

### 1. Seat Locking and Double Booking Prevention

The central system-design problem is preventing two users from booking the same seat at the same time.

The backend handles this with a seat locking service and per-show synchronization. Lock operations for the same show are serialized, while different shows can still be processed independently.

```text
User A -> lock A1,A2 -> show-level critical section -> seats become Locked
User B -> lock A1,A2 -> re-check seats -> receives unavailable response
```

This gives the system the right balance: strong correctness for a show under contention without globally blocking all booking traffic.

### 2. Timed Seat Holds

When a user selects seats, those seats are not immediately booked. They are temporarily locked while the user completes payment.

If the user abandons the flow, a background worker releases the seats after the lock expires.

```text
Available -> Locked -> Booked
              |
              +-> Available again if lock expires
```

### 3. Booking State Machine

Bookings are represented as explicit lifecycle states instead of scattered boolean flags.

```text
Pending -> PaymentPending -> Success
   |            |
   |            +-> PaymentFailed
   |
   +-> Cancelled
   +-> Expired

PaymentPending -> SuccessPartial
```

This makes invalid state transitions easier to reject and reason about.

### 4. Background Workers

The backend runs time-aware background jobs using Tokio:

- lock expiration sweep
- payment timeout sweep
- queue processing loop

These jobs keep the system consistent even when users close their browser or payment is never completed.

### 5. Queue-Oriented Booking Flow

The backend includes a queue service for high-contention show booking. This models how real ticketing systems handle spikes when many users attempt to book the same show at the same time.

### 6. Idempotent Payment Initiation

Payment initiation is designed to be retry-safe. The backend supports idempotency keys so duplicate requests do not create duplicate payment records.

This matters because payment flows often involve refreshes, retries, double-clicks, and network failures.

### 7. Repository Abstraction

The backend service layer depends on repository traits, not concrete database implementations.

```text
API handlers -> service layer -> repository traits -> Postgres / in-memory repositories
```

This makes core business logic testable and keeps persistence details out of the domain layer.

## Rust Backend Highlights

The Rust backend is organized as a Cargo workspace:

```text
backend/
  crates/
    common/                 shared config, errors, API response types
    domain/                 pure domain models and enums
    repository/             repository traits
    repository-inmemory/    in-memory repository implementations
    repository-postgres/    PostgreSQL repository implementations
    service/                business logic and orchestration
    api/                    Axum routes, handlers, app state, startup
```

Key Rust engineering choices:

- Axum for async HTTP APIs.
- Tokio for async runtime and background tasks.
- SQLx for PostgreSQL persistence.
- Trait-based repositories for clean dependency inversion.
- Strong domain modeling with Rust enums and structs.
- Centralized error handling through typed application errors.
- Clear separation between HTTP handlers, services, repositories, and domain models.
- JWT-based auth for users and admins.
- Admin bootstrap user seeded from environment variables.

Important backend concepts implemented:

- show management
- movie and venue management
- seat layout generation
- seat locking and expiry
- booking lifecycle management
- payment initiation and completion
- queue management
- audit logs
- compensation logs
- analytics for occupancy and revenue

## Next.js Frontend Highlights

The frontend is built with Next.js App Router and TypeScript.

Key frontend engineering choices:

- Typed API client using Axios and shared TypeScript interfaces.
- Auth-protected user and admin pages.
- App Router page organization.
- CSS Modules for scoped styling.
- Reusable UI components for buttons, inputs, selects, modals, badges, loading states, and booking components.
- Responsive dark cinema-style interface.
- Interactive seat map with multiple layout types.
- Clean booking confirmation ticket UI.

Important frontend flows:

- user registration and login
- movie browsing
- movie detail and showtime selection
- seat quantity selection
- seat selection
- payment
- confirmation ticket
- my bookings
- admin dashboard
- admin show creation
- admin booking inspection

## Domain Model

| Model | Purpose |
| --- | --- |
| Movie | Film metadata such as title, genre, language, rating, poster, and duration |
| Venue | Theatre information, city, address, screens, and amenities |
| Show | A movie screening at a venue and screen for a specific time |
| Seat | A physical seat for a show, with type and status |
| Booking | User's selected seats and lifecycle state |
| Payment | Payment intent, gateway state, and amount |
| QueueEntry | User's position in a high-contention booking queue |
| CompensationLog | Record for partial success and refund-like recovery scenarios |
| AuditLog | Timeline of booking and admin events |

## Seat Layouts and Pricing

The project supports different show experiences:

- Normal: standard, comfort, and recliner sections.
- Luxe: recliner-only seating.
- IMAX: premium large-format layout.

Seat pricing is based on seat type:

- Standard: base price
- Comfort: higher multiplier
- Recliner: premium multiplier

The seat layout UI separates rows and sections visually, shows the screen at the bottom, and supports selected, locked, booked, and available states.

## API Overview

Representative API areas:

```text
Auth
  POST /auth/register
  POST /auth/login
  POST /auth/refresh

Movies and shows
  GET  /movies
  GET  /movies/:movie_id
  GET  /movies/:movie_id/shows
  GET  /shows
  GET  /shows/:show_id
  GET  /shows/:show_id/seats
  GET  /shows/:show_id/availability

Booking
  POST   /shows/:show_id/seats/lock
  GET    /bookings/:booking_id
  POST   /bookings/:booking_id/cancel
  DELETE /bookings/:booking_id/lock
  GET    /bookings/user/:user_id

Payment
  POST /bookings/:booking_id/payment/initiate
  POST /payments/mock/pay

Admin
  POST /admin/shows
  GET  /admin/bookings
  GET  /admin/audit
  POST /admin/venues
  POST /admin/movies
```

## Architecture

```text
Browser
  |
  v
Next.js frontend
  |
  v
Rust Axum API
  |
  +--> Auth service
  +--> Movie / venue / show service
  +--> Seat locking service
  +--> Booking service
  +--> Payment service
  +--> Queue service
  |
  v
Repository traits
  |
  +--> PostgreSQL repositories
  +--> In-memory repositories

Background workers:
  - expired seat lock cleanup
  - expired payment cleanup
  - queue processor
```

## Local Development

### Prerequisites

- Docker and Docker Compose
- Node.js 20+
- Rust toolchain
- PostgreSQL if running backend outside Docker

### Docker setup

```bash
docker compose build
docker compose up
```

Services:

- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- Metrics: http://localhost:9000
- PostgreSQL: localhost:5432
- Redis: localhost:6379

Default admin credentials:

```text
Email: admin@bookmyshow.com
Password: Admin@123
```

These can be overridden with `ADMIN_EMAIL` and `ADMIN_PASSWORD`.

### Frontend only

```bash
cd frontend
npm install
npm run dev
```

### Backend checks

```bash
cargo check --manifest-path backend/Cargo.toml -p api
cargo fmt --manifest-path backend/Cargo.toml --all
```

### Frontend checks

```bash
cd frontend
./node_modules/.bin/tsc --noEmit
npm run build
```

## Environment Variables

Common variables used by Docker Compose:

```text
JWT_SECRET
ADMIN_EMAIL
ADMIN_PASSWORD
DATABASE_URL
REDIS_URL
STRIPE_SECRET_KEY
STRIPE_WEBHOOK_SECRET
NEXT_PUBLIC_API_BASE_URL
NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY
OTLP_ENDPOINT
```

## What This Shows on a Resume

This project is useful to highlight because it is not a simple CRUD app. It demonstrates:

- full-stack product thinking
- concurrent booking and seat-locking design
- Rust backend engineering
- async services and background jobs
- state-machine driven business logic
- repository pattern and layered architecture
- PostgreSQL persistence
- admin and user workflows
- Dockerized local deployment
- typed Next.js frontend development
- UI polish for a real consumer workflow

Suggested resume bullet:

```text
Built a full-stack cinema booking platform using Rust, Axum, Next.js, PostgreSQL, and Docker, featuring timed seat locks, booking state machines, payment flow, admin show management, venue/movie linking, and concurrent double-booking prevention.
```

## Future Improvements

- Add production Stripe webhook handling.
- Add email or SMS ticket delivery.
- Add real-time seat updates with WebSockets.
- Add richer movie and venue creation flows in admin.
- Add screenshot gallery and hosted demo video.
- Add more integration tests around payment and queue edge cases.
- Add observability dashboards for metrics and traces.

## Status

The project is actively being refined. Core booking, payment, admin, movie, venue, and seat-selection flows are implemented, with ongoing UI and portfolio polish.
