# BookMyShow — Production Readiness Plan

> Date: 2026-05-04  
> Status snapshot: ~70% MVP-complete. Core booking flow works end-to-end. Critical gaps: persistence, auth, payments, deployment.

---

## Current State Summary

| Layer | Done | Partial | Missing |
|-------|------|---------|---------|
| Backend API routes | ✅ All endpoints | — | — |
| Backend services (booking, payment, queue, seat lock) | ✅ Full impl | — | — |
| Backend domain models + state machines | ✅ All entities | — | — |
| Backend persistence | ❌ | — | Real DB (in-memory only) |
| Backend auth | — | ⚠️ UUID header only | JWT, registration, login |
| Backend distributed locking | — | ⚠️ Single-server mutex | Redis-based for multi-node |
| Backend email | ❌ | — | No email service wired |
| Backend payment | — | ⚠️ Mock gateway only | Real Stripe/payment gateway |
| Backend tests | — | ⚠️ Integration 80%, unit 20% | Unit tests for services/domain |
| Frontend pages | ✅ All pages | — | — |
| Frontend API integration | ✅ All endpoints | — | — |
| Frontend auth UI | ❌ | — | Login/register pages |
| Frontend seat pricing by type | — | ⚠️ Display only | Type-based multiplier (1×/1.5×/2×) |
| Frontend print/PDF ticket | — | ⚠️ Display only | Print CSS / PDF export |
| Deployment | ❌ | — | Docker, CI/CD, env templates |

---

## Pending Items (Priority Order)

### PHASE 1 — Foundation (Blockers for anything real)

#### 1.1 Database Layer (PostgreSQL)
- [ ] Add `sqlx` or `diesel` to backend workspace
- [ ] Write SQL migrations for all tables: `users`, `shows`, `seats`, `bookings`, `payments`, `seat_locks`, `queue_entries`, `compensation_logs`
- [ ] Implement `repository-postgres` crate (swap in-memory impls with real DB queries)
- [ ] Wire DB pool into `AppState`
- [ ] Add `DATABASE_URL` to config

#### 1.2 User Registration & Login ✅ DONE
- [x] **Backend:** Add `POST /auth/register` (email + password, bcrypt hash)
- [x] **Backend:** Add `POST /auth/login` (verify password → issue JWT)
- [x] **Backend:** Add `POST /auth/refresh` (refresh token rotation)
- [x] **Backend:** Replace `X-User-Id` UUID header with JWT Bearer middleware (backward-compat fallback kept for tests)
- [x] **Backend:** Store `user_id`, `email`, `password_hash`, `role` in in-memory repo (DB deferred)
- [x] **Frontend:** Create `/login` page (email + password form)
- [x] **Frontend:** Create `/register` page (email + password + confirm)
- [x] **Frontend:** Add `AuthContext` + `useAuth` hook (store JWT in localStorage, auto-refresh)
- [x] **Frontend:** Add protected route wrapper (redirect to `/login` if no token) — `useRequireAuth` + `useRequireAdmin` hooks guard all protected pages
- [x] **Frontend:** Auto-logout on token expiry (401 → clear tokens + redirect)

#### 1.3 Admin Auth Hardening ✅ DONE
- [x] **Backend:** Issue admin JWT via `POST /admin/auth/login`
- [x] **Backend:** Admin user seeded at startup (`admin@bookmyshow.com` / `Admin@123` or env vars `ADMIN_EMAIL`/`ADMIN_PASSWORD`)
- [x] **Frontend:** `/admin/login` page
- [x] **Frontend:** Store admin JWT in `bms_admin_token`, sent as `X-Admin-Token` header

---

### PHASE 2 — Core Feature Completeness

#### 2.1 Seat Pricing by Type ✅ DONE
- [x] **Backend:** Add `price_multiplier` per seat type in show creation (Standard=1.0×, Premium=1.5×, Recliner=2.0×)
- [x] **Backend:** Recalculate `total_amount` on booking based on seat types
- [x] **Frontend:** Show per-seat price in seat grid tooltip/hover
- [x] **Frontend:** Break down price in `SeatSelectionPanel` (e.g. "2× Standard ₹200 + 1× Premium ₹300 = ₹700")

#### 2.2 Real Payment Gateway ✅ DONE
- [x] **Backend:** Integrate Stripe (or Razorpay for India)
  - [x] `POST /bookings/:id/payment/initiate` → create Stripe PaymentIntent
  - [x] `POST /payments/webhook` → Stripe webhook handler (verify signature)
  - [x] Idempotency key already supported — wire to Stripe's idempotency header
  - [x] Handle `payment_intent.succeeded`, `payment_intent.payment_failed` events
- [x] **Frontend:** Replace mock card form with Stripe Elements (`@stripe/react-stripe-js`)
- [x] **Frontend:** Remove mock-gateway page/flow

#### 2.3 Email Notifications ✅ DONE
- [x] **Backend:** Integrate email provider (Resend / SendGrid / SMTP)
- [x] **Backend:** Send on:
  - Booking confirmed → confirmation + e-ticket PDF
  - Payment failed → retry link
  - Booking cancelled → cancellation notice + refund ETA
  - Lock expiry warning (5 min before) → nudge to complete payment
- [x] **Backend:** HTML email templates (booking ID, show name, seats, total, QR code stub)

#### 2.4 Print / PDF Ticket ✅ DONE
- [x] **Frontend:** Add print CSS to `TicketDisplay` (hide nav, show full ticket)
- [x] **Frontend:** "Download PDF" button on `/bookings/:id/confirmed` (use `window.print()` or `jsPDF`)
- [ ] **Backend (optional):** `GET /bookings/:id/ticket.pdf` endpoint returning server-rendered PDF

---

### PHASE 3 — Reliability & Production Hardening

#### 3.1 Atomic Transactions ✅ DONE (DB transaction deferred)
- [x] **Backend:** Repository-level atomic lock creation + booking insert rollback added; single DB transaction deferred until database setup
- [x] **Backend:** Payment callback + booking status update hardened with idempotent callback handling and compensation audit logging; DB transaction deferred until database setup
- [x] **Backend:** Compensating transaction on partial failure (rollback seat lock if booking insert fails)

#### 3.2 Distributed Locking (Multi-server) ✅ DONE
- [x] **Backend:** Add Redis as optional dep (feature flag `distributed-lock`)
- [x] **Backend:** Replace per-show `tokio::Mutex` with Redis `SET NX PX` lock per show_id when `REDIS_URL` + feature are enabled
- [x] **Backend:** Lease renewal for long-running lock holders
- [x] **Backend:** Fallback to in-process mutex when Redis unavailable (single-server mode)

#### 3.3 Audit Logging ✅ DONE
- [x] **Backend:** Fully wire `CompensationLog` — write entry on every state transition (lock, book, pay, refund, cancel)
- [x] **Backend:** `GET /admin/audit?booking_id=&user_id=` endpoint
- [x] **Frontend:** Show audit trail on admin booking detail page

#### 3.4 Input Validation Hardening ✅ DONE
- [x] **Backend:** Validate email format on registration
- [x] **Backend:** Enforce max seats per booking at API level (already in service, add to route docs)
- [x] **Backend:** Validate `show_id` UUID format before DB query (avoid malformed ID errors)
- [x] **Frontend:** Client-side form validation (email regex, password strength, card number Luhn)

#### 3.5 Rate Limiting Expansion ✅ DONE
- [x] **Backend:** Extend rate limiting to payment initiation (prevent payment spam)
- [x] **Backend:** Global IP-based rate limit on auth endpoints (prevent brute force)
- [x] **Backend:** Return `Retry-After` header on 429 responses

---

### PHASE 4 — UX Polish

#### 4.1 Accessibility
- [x] **Frontend:** Add `aria-label` to all icon buttons (seat cells, close buttons, nav)
- [x] **Frontend:** Keyboard navigation for seat grid (arrow keys, space to select)
- [x] **Frontend:** Focus management in modals (trap focus, restore on close)
- [ ] **Frontend:** Color contrast audit (WCAG 2.1 AA minimum)
- [ ] **Frontend:** Screen reader announcements for seat status changes and timer

#### 4.2 Movie / Show Metadata
- [ ] **Backend:** Add `movie` table: title, genre, language, duration, poster_url, rating, description
- [ ] **Backend:** Link `show` to `movie` (show is a screening of a movie)
- [ ] **Backend:** `GET /movies` — browse movies
- [ ] **Backend:** `GET /movies/:id/shows` — all shows for a movie
- [ ] **Frontend:** `/movies` home page (movie posters, filter by genre/language/rating)
- [ ] **Frontend:** `/movies/:id` — movie detail with all upcoming shows
- [ ] **Frontend:** Update `/` to be movie-first browsing, not show-first

#### 4.3 Venue / Theatre Metadata  
- [ ] **Backend:** Add `venue` table: name, address, city, screen count, amenities
- [ ] **Backend:** Link `show` to `venue` + `screen`
- [ ] **Frontend:** Display full venue address on show/booking pages
- [ ] **Frontend:** Filter shows by city

#### 4.4 My Bookings Enhancements
- [ ] **Frontend:** Show QR code stub on each booking card
- [ ] **Frontend:** "Download Ticket" button per booking
- [ ] **Frontend:** Pagination (currently loads all bookings)

---

### PHASE 5 — Deployment & DevOps

#### 5.1 Docker
- [ ] `backend/Dockerfile` — multi-stage Rust build (builder + slim runtime)
- [ ] `frontend/Dockerfile` — Next.js production build
- [ ] `docker-compose.yml` — backend + frontend + postgres + redis services
- [ ] `.env.example` files for both backend and frontend

#### 5.2 CI/CD (GitHub Actions)
- [ ] `.github/workflows/ci.yml`:
  - Rust: `cargo test`, `cargo clippy`, `cargo fmt --check`
  - Next.js: `npm run lint`, `npm run build`, type check
- [ ] `.github/workflows/deploy.yml` (optional):
  - Build and push Docker images to registry
  - Deploy to target (Fly.io / Railway / ECS / etc.)

#### 5.3 Environment Config
- [ ] `backend/.env.example` — `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET`, `ADMIN_TOKEN`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `EMAIL_API_KEY`
- [ ] `frontend/.env.example` — `NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY`
- [ ] Secrets management guidance in README

#### 5.4 Observability
- [ ] **Backend:** Structured logs already exist — ship to Loki / CloudWatch / Datadog
- [ ] **Backend:** Add OpenTelemetry tracing (trace booking flow end-to-end)
- [ ] **Backend:** Prometheus metrics endpoint (`/metrics`) — booking rate, lock contention, payment success rate
- [ ] **Frontend:** Add error boundary with Sentry integration

---

### PHASE 6 — Nice-to-Have (Post-MVP)

- [ ] OAuth login (Google / GitHub)
- [ ] OTP-based phone login
- [ ] Waitlist for sold-out shows (email when seat frees up)
- [ ] Reviews & ratings per show
- [ ] Wishlist / saved shows
- [ ] Multi-language (i18n)
- [ ] Dark mode toggle
- [ ] Mobile PWA (service worker, offline ticket cache)
- [ ] Loyalty points / rewards system
- [ ] Group bookings (send invite link to friends)
- [ ] Dynamic pricing (surge pricing based on demand)

---

## Item Count by Phase

| Phase | Items | Priority |
|-------|-------|----------|
| Phase 1 — Foundation | 18 | Critical |
| Phase 2 — Core Features | 16 | High |
| Phase 3 — Hardening | 12 | High |
| Phase 4 — UX Polish | 15 | Medium |
| Phase 5 — DevOps | 12 | Medium |
| Phase 6 — Nice-to-Have | 10 | Low |
| **Total** | **83** | — |
