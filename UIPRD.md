# BookMyShow — UI Product Requirements Document

> **Version:** 1.0
> **Date:** 2026-04-30
> **Status:** Draft
> **Language:** React + TypeScript (or Next.js)
> **Backend Base URL:** `http://localhost:8080`

---

## Table of Contents

1. [Vision & Design Language](#1-vision--design-language)
2. [Backend Contract](#2-backend-contract)
3. [Page Architecture](#3-page-architecture)
4. [Page Specifications](#4-page-specifications)
5. [Component Library](#5-component-library)
6. [User Flows](#6-user-flows)
7. [Admin Flows](#7-admin-flows)
8. [Error Handling & Edge Cases](#8-error-handling--edge-cases)
9. [Tech Stack & Architecture](#9-tech-stack--architecture)
10. [Out of Scope](#10-out-of-scope)

---

## 1. Vision & Design Language

### 1.1 Design Philosophy

This is a **cinema-grade booking application** — not a SaaS dashboard, not a generic ticket platform. The UI must feel like sitting in a real theatre, with the seat grid as the centerpiece and the booking flow as the primary experience. Every screen should reinforce the drama and anticipation of going to the movies.

Design principles:
- **Cinematic**: Dark theme with rich, warm accents. The seat map is the hero element.
- **Tactile**: Seats are clickable, animated, and give clear visual feedback on selection, locking, and booking.
- **Urgency-aware**: Lock timers are always visible. The UI communicates time pressure without panic.
- **Honest**: Every state — available, locked by you, locked by others, booked — is visually distinct and unambiguous.
- **No AI/SaaS aesthetics**: No floating abstract blobs, no gradient-heavy SaaS hero sections, no robot-generated stock imagery. The aesthetic is purposeful and grounded.

### 1.2 Color System

| Token | Hex | Usage |
|-------|-----|-------|
| `--bg-primary` | `#0D0D0F` | Page background |
| `--bg-surface` | `#161619` | Cards, panels, seat grid background |
| `--bg-elevated` | `#1E1E23` | Modals, popovers, hover states |
| `--text-primary` | `#F5F5F7` | Headings, primary text |
| `--text-secondary` | `#9CA3AF` | Descriptions, labels, metadata |
| `--text-muted` | `#6B7280` | Placeholders, disabled text |
| `--accent-gold` | `#F5A623` | Primary CTA, selected seats, brand accent |
| `--accent-gold-dim` | `#B8860B` | Hover state for gold |
| `--seat-available` | `#22C55E` | Available seats (green fill) |
| `--seat-locked-you` | `#F5A623` | Seats locked by current user (gold) |
| `--seat-locked-other` | `#EF4444` | Seats locked by others (red) |
| `--seat-booked` | `#6B7280` | Booked seats (grey) |
| `--seat-selected` | `#F5A623` | Currently hovered/selected seats (gold outline) |
| `--seat-premium` | `#A855F7` | Premium row indicator |
| `--seat-recliner` | `#06B6D4` | Recliner row indicator |
| `--danger` | `#EF4444` | Errors, cancel actions |
| `--info` | `#3B82F6` | Informational states |

### 1.3 Typography

| Role | Font | Weight | Size |
|------|------|--------|------|
| Page heading | Inter | 700 | 28–36px |
| Section heading | Inter | 600 | 20–24px |
| Card title | Inter | 600 | 16px |
| Body | Inter | 400 | 14–16px |
| Caption/meta | Inter | 400 | 12px |
| Timer/countdown | JetBrains Mono | 700 | 20–32px |
| Seat labels | JetBrains Mono | 500 | 12px |

### 1.4 Spacing System

Base unit: `4px`. Spacing scale: `4, 8, 12, 16, 20, 24, 32, 40, 48, 64px`.

### 1.5 Motion

| Interaction | Animation |
|-------------|-----------|
| Seat hover | `scale(1.1)`, 150ms ease-out |
| Seat select | `scale(0.95)` then bounce back, 200ms spring |
| Seat lock acquired | Subtle pulse glow, 400ms |
| Lock timer tick | No animation — stable display, just numeric countdown |
| Page transitions | Fade + slide-up, 250ms ease-out |
| Modal open | Fade-in backdrop 200ms, modal scale from 0.95→1, 200ms ease-out |
| Toast notification | Slide in from top-right, auto-dismiss 4s |
| Loading skeleton | Shimmer effect, 1.5s infinite |

### 1.6 Screen Layout Rhythm

- **Home/Browse**: Dense grid of show cards — focus on content discovery
- **Seat Selection**: Full-bleed seat grid is the hero — minimal chrome around it
- **Payment**: Centered, focused — the timer and payment form are all that matters
- **Confirmation**: Celebratory but restrained — booking details prominent, next steps clear
- **Admin**: Data-dense, table-heavy — analytics and management tools

---

## 2. Backend Contract

### 2.1 Base URL & Auth

- **Base URL**: `http://localhost:8080`
- **User Auth**: All user requests include header `X-User-Id: <uuid>`. On first visit, generate a UUID and store in `localStorage`.
- **Admin Auth**: Admin requests include header `X-Admin-Token: <token>`. Default token from env: `admin-secret` (for dev).
- **Response envelope**: All responses follow `{ success: true, data: {...}, timestamp: "..." }`. On error: `{ success: false, error: { code: "...", message: "...", details: {...} }, timestamp: "..." }`.

### 2.2 API Endpoints

#### Public (no auth required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/shows` | List all shows |
| `GET` | `/shows/:show_id` | Get show details |
| `GET` | `/shows/:show_id/seats?page=1&limit=100` | Get seat layout |
| `GET` | `/shows/:show_id/availability` | Get availability summary |

#### User (requires `X-User-Id` header)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/shows/:show_id/seats/lock` | Lock seats |
| `POST` | `/bookings/:booking_id/extend-lock` | Extend lock TTL |
| `DELETE` | `/bookings/:booking_id/lock` | Release lock |
| `GET` | `/bookings/:booking_id` | Get booking details |
| `POST` | `/bookings/:booking_id/cancel` | Cancel booking |
| `POST` | `/bookings/:booking_id/payment/initiate` | Initiate payment |
| `GET` | `/bookings/user/:user_id` | User's booking history |
| `GET` | `/payments/:payment_id` | Get payment status |
| `POST` | `/shows/:show_id/queue/join` | Join show queue |
| `GET` | `/queue/:queue_id/status` | Poll queue status |
| `DELETE` | `/queue/:queue_id` | Leave queue |

#### Admin (requires `X-Admin-Token` header)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/admin/shows` | Create show |
| `DELETE` | `/admin/shows/:show_id` | Cancel show + refund |
| `GET` | `/admin/shows/:show_id/analytics` | Show analytics |
| `POST` | `/admin/shows/:show_id/seats/:seat_id/override` | Force-release seat |
| `GET` | `/admin/bookings` | List all bookings |
| `POST` | `/admin/payments/:payment_id/refund` | Issue refund |

#### Internal (mock gateway — called by backend on payment callback)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/mock-gateway/pay` | Trigger mock payment |

### 2.3 Key Request/Response Shapes

#### Lock Seats

```
POST /shows/:show_id/seats/lock
Header: X-User-Id: <uuid>
Body: { "seat_ids": ["uuid-1", "uuid-2"] }

Response 201:
{
  "booking_id": "uuid-booking",
  "lock_id": "uuid-lock",
  "show_id": "uuid-show",
  "seat_ids": ["uuid-1", "uuid-2"],
  "total_amount": 450.00,
  "expires_at": 1746032400,   // Unix timestamp (seconds)
  "status": "Pending"
}
```

#### Extend Lock

```
POST /bookings/:booking_id/extend-lock
Header: X-User-Id: <uuid>

Response: same as lock seats response, with updated expires_at
```

#### Payment Initiate

```
POST /bookings/:booking_id/payment/initiate
Header: X-User-Id: <uuid>

Response:
{
  "payment_id": "uuid-payment",
  "payment_intent_id": "uuid-intent",
  "amount": 450.00,
  "gateway_name": "mock",
  "status": "pending"
}
```

#### Seat Layout

```
GET /shows/:show_id/seats

Response:
{
  "show_id": "uuid",
  "seats": [
    {
      "seat_id": "uuid",
      "seat_number": "A1",
      "row_label": "A",
      "seat_type": "Premium",    // "Standard" | "Premium" | "Recliner"
      "status": "Available",     // "Available" | "Locked" | "Booked"
      "lock_expires_at": null     // null if not locked, Unix timestamp otherwise
    },
    ...
  ],
  "page": 1,
  "limit": 100
}
```

#### Queue Status

```
GET /queue/:queue_id/status

Response:
{
  "queue_id": "uuid",
  "status": "Locked",           // "Waiting" | "Processing" | "Locked" | "Conflict" | "Expired"
  "position": 1,
  "booking_id": "uuid",         // set when status = "Locked"
  "lock_id": "uuid",
  "conflict_seats": null        // array of seat_ids if status = "Conflict"
}
```

---

## 3. Page Architecture

| # | Route | Purpose | Access |
|---|-------|---------|--------|
| 1 | `/` | Home — browse all shows | Public |
| 2 | `/shows/:showId` | Seat selection — interactive seat map | User |
| 3 | `/bookings/:bookingId` | Booking details — lock status + timer | User |
| 4 | `/bookings/:bookingId/payment` | Payment screen — initiate + complete | User |
| 5 | `/bookings/:bookingId/confirmed` | Booking confirmation | User |
| 6 | `/my-bookings` | User's booking history | User |
| 7 | `/admin` | Admin dashboard — overview | Admin |
| 8 | `/admin/shows/new` | Create show form | Admin |
| 9 | `/admin/shows/:showId` | Show analytics + seat override | Admin |

---

## 4. Page Specifications

### 4.1 Home — Browse Shows (`/`)

**Purpose**: Display all available shows so users can select one to book seats.

**Layout**:
- Header: App name "BookMyShow", nav to My Bookings, Admin link (if admin token present)
- Filter bar: Optional date filter, search by show name
- Show grid: 3-column responsive grid of show cards
- Empty state: Illustration + "No shows available right now" message

**Show Card** (per show):
- Movie/show name (large, bold)
- Theatre name + Screen number
- Date and time (formatted: "Fri, May 1 · 2:00 PM")
- Availability badge: "XX seats available" (green if >50%, yellow if >20%, red if ≤20%)
- Price: "₹XXX per seat"
- CTA: "Select Seats" button → navigates to `/shows/:showId`

**Behavior**:
- On page load: `GET /shows` → display shows sorted by `start_time`
- Each card shows availability from `GET /shows/:showId/availability`

---

### 4.2 Seat Selection (`/shows/:showId`)

**Purpose**: The core interaction — let user browse the seat map and lock seats.

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ [Back]  Show Name · Theatre Name · Screen X               │
│          Fri, May 1 · 2:00 PM                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│              ┌────────── SCREEN ──────────                 │
│                                                             │
│  A  [seat][seat][seat][seat][seat][seat][seat][seat]...    │
│  B  [seat][seat][seat][seat][seat][seat][seat][seat]...    │
│  C  ...                                                    │
│                                                             │
│  [Legend: Available · Your Selection · Taken]             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Selected (0): —                          Total: ₹0        │
│ [Lock Seats — 5 min timer starts on lock]                    │
└─────────────────────────────────────────────────────────────┘
```

**Seat Grid Rules**:
- Render seats in rows, left to right per row
- Each seat is a clickable element showing the seat number (A1, A2, etc.)
- Row labels on the left side
- Screen indicator above the grid (a horizontal line or subtle curve)

**Seat Visual States**:
| State | Fill | Border | Cursor | Clickable |
|-------|------|--------|--------|-----------|
| Available | `#22C55E` green | none | pointer | Yes |
| Selected (by current user) | `#F5A623` gold | gold glow | pointer | Yes (deselect) |
| Locked by others | `#EF4444` red | none | not-allowed | No |
| Booked | `#6B7280` grey | none | not-allowed | No |

**Seat Type Indicators**:
- Premium row: Small badge or tinted header row in `#A855F7` purple
- Recliner row: Small badge or tinted header row in `#06B6D4` cyan
- Standard row: No indicator (default)

**Seat Tooltip** (on hover): Show seat number, seat type, price modifier.

**Behavior**:
1. On page load: `GET /shows/:showId/seats` → render full seat grid. Poll every 5s for updates (or use WebSocket if time permits).
2. User clicks seats → add to selected seats array (max 10). Show running total.
3. If user clicks a seat locked by others → show toast: "Seat X is currently held by another user".
4. If user clicks a booked seat → show toast: "Seat X is already booked".
5. "Lock Seats" button:
   - Disabled if 0 seats selected
   - On click: `POST /shows/:showId/seats/lock` with selected seat_ids
   - On success (201): Navigate to `/bookings/:bookingId` with lock response data
   - On 409 Conflict: Highlight conflicting seats in red, show toast listing them
   - On 429 Rate Limit: Show toast "Too many requests. Please wait a moment."
   - On 400 Bad Request: Show error message (e.g., "Maximum 10 seats per booking")

**Side Panel** (or bottom sheet on mobile):
- "Your Selection" list: Shows seat numbers + price per seat
- Running total amount
- "Lock Seats" CTA button with lock icon
- If lock is already active for this show (checked via localStorage): show "You have active locks" banner

---

### 4.3 Booking Details (`/bookings/:bookingId`)

**Purpose**: Show the user's active booking with lock countdown and action buttons.

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Your Booking                                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⏱ Lock expires in                                          │
│     04:32                                                   │
│     (6 seats · ₹1,250.00)                                  │
│                                                             │
│  Show: Avengers: Endgame                                   │
│  Theatre: PVR Nexus · Screen 1                             │
│  Time: Fri, May 1 · 2:00 PM                                │
│                                                             │
│  Seats: A1, A2, A3, A4, A5, A6                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [Extend Lock (+2 min)]  [Proceed to Payment]  [Cancel]    │
└─────────────────────────────────────────────────────────────┘
```

**Lock Timer**:
- Countdown from `expires_at` (Unix timestamp) → display as `MM:SS`
- Color: Green (>2 min), Yellow (30s–2 min), Red (<30s — pulsing)
- When timer hits 0: Auto-detect via polling `GET /bookings/:bookingId`. If status changed to `Expired`, show modal: "Your lock has expired. Seats have been released."
- Every 10s: Poll `GET /bookings/:bookingId` to check if lock status changed (expired by backend).

**Actions**:
- **Extend Lock**: `POST /bookings/:bookingId/extend-lock`. Available up to 2 times total (track in localStorage or UI counter). Show remaining extensions. Button disabled + tooltip when max reached.
- **Proceed to Payment**: Navigate to `/bookings/:bookingId/payment`
- **Cancel**: `POST /bookings/:bookingId/cancel`. Show confirmation dialog first: "Cancel this booking? Your seats will be released." On success, navigate back to show's seat selection.

**Status Display**:
- `Pending`: Show lock timer + all three actions
- `PaymentPending`: Show "Payment in progress" banner + cancel option
- `Success`: Redirect to `/bookings/:bookingId/confirmed`
- `Expired`, `Cancelled`: Show status card with message and "Browse Shows" CTA

---

### 4.4 Payment (`/bookings/:bookingId/payment`)

**Purpose**: Initiate and complete payment within the lock window.

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Payment                                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⏱ Complete payment before                                 │
│     03:45                                                   │
│                                                             │
│  Order Summary                                              │
│  ──────────────────                                         │
│  Seats: A1, A2, A3, A4, A5, A6 (Premium ×2, Standard ×4)  │
│  Amount: ₹1,250.00                                         │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│  Total: ₹1,250.00                                           │
│                                                             │
│  Payment Method                                            │
│  ┌──────────────────────────────────────┐                  │
│  │ [Credit/Debit Card icon]  Card      │                  │
│  └──────────────────────────────────────┘                  │
│                                                             │
│  Card Number                                                │
│  ┌──────────────────────────────────────┐                  │
│  │ 4242 4242 4242 4242                  │                  │
│  └──────────────────────────────────────┘                  │
│                                                             │
│  Expiry          CVV                                        │
│  ┌───────────┐  ┌───────────┐                               │
│  │ MM / YY   │  │ ***       │                               │
│  └───────────┘  └───────────┘                               │
│                                                             │
│  ┌──────────────────────────────────────┐                  │
│  │     💳 Pay ₹1,250.00                 │                  │
│  └──────────────────────────────────────┘                  │
│                                                             │
│  [Cancel Payment]                                          │
└─────────────────────────────────────────────────────────────┘
```

**Flow**:
1. On page load: `POST /bookings/:bookingId/payment/initiate` → get `payment_id`, `payment_intent_id`
2. Show payment form with pre-filled amount
3. User enters card details (for mock: any valid-format card number works)
4. "Pay" button: `POST /mock-gateway/pay` with `payment_intent_id`, `amount`, `card_last4`, `simulate_failure: false`
5. The mock gateway has a 2s delay, then:
   - 80% success → booking confirmed → redirect to `/bookings/:bookingId/confirmed`
   - 20% failure → show error "Payment failed. Please try again." with retry button. Seats remain locked.
6. While waiting: Show spinner on Pay button, disable form.
7. If lock expires during payment: Backend callback fails validation. Show "Lock expired" modal → redirect to seat selection.

**Lock Timer**: Always visible at top. Same color-coding as booking page.

**Mock Card Numbers** (for testing):
- Success: `4242 4242 4242 4242` (Visa test)
- Fail: Any other valid-format number → triggers failure (mock has 20% random failure rate; use `simulate_failure: true` to force it)

---

### 4.5 Booking Confirmed (`/bookings/:bookingId/confirmed`)

**Purpose**: Celebrate the confirmed booking and provide ticket details.

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ 🎉 Booking Confirmed!                                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─── TICKET ───────────────────────────────────────────┐   │
│  │                                                      │   │
│  │  AVENGERS: ENDGAME                                   │   │
│  │  PVR Nexus · Screen 1                                │   │
│  │  Fri, May 1, 2026 · 2:00 PM                          │   │
│  │                                                      │   │
│  │  ─────────────────────────────────                   │   │
│  │                                                      │   │
│  │  SEATS                                              │   │
│  │  A1 · A2 · A3 · A4 · A5 · A6                       │   │
│  │                                                      │   │
│  │  ─────────────────────────────────                   │   │
│  │                                                      │   │
│  │  AMOUNT PAID  ₹1,250.00                             │   │
│  │  BOOKING ID   BMS-XXXXXXXX                          │   │
│  │  PAYMENT ID   XXXXXXXX                             │   │
│  │                                                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  [Browse More Shows]   [View My Bookings]                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Behavior**:
- Auto-fetch `GET /bookings/:bookingId` to display confirmed details
- "Print Ticket" button: Opens print dialog with a formatted ticket layout
- Confetti animation: Subtle CSS confetti burst on page load (1x, not excessive)

---

### 4.6 My Bookings (`/my-bookings`)

**Purpose**: Show user's booking history.

**Layout**:
- Header: "My Bookings", back nav
- Filter tabs: All | Confirmed | Cancelled | Pending
- Booking list: Cards showing show name, seats, status badge, date, amount
- Empty state: "No bookings yet. Browse shows to get started."

**Booking Card**:
- Show name + theatre
- Date/time
- Seat count and numbers
- Status badge: Confirmed (green), Cancelled (red), Pending (yellow), Expired (grey)
- Payment status if applicable
- "View Details" → `/bookings/:bookingId`

**Behavior**:
- On page load: `GET /bookings/user/:userId` (userId from localStorage)
- Display bookings sorted by `created_at` descending

---

### 4.7 Admin Dashboard (`/admin`)

**Purpose**: Admin overview with show analytics and booking management.

**Layout**:
- Header: "Admin Panel", app name
- Tab navigation: Shows | Bookings | Refunds
- Stats cards row: Total Shows, Total Bookings Today, Revenue Today, Active Locks
- Shows table: Name, Screen, Date, Seats Booked, Revenue, Actions
- Each row has "Analytics" link → `/admin/shows/:showId`

---

### 4.8 Admin — Create Show (`/admin/shows/new`)

**Purpose**: Form for admin to create a new show with seat layout.

**Form Fields**:
| Field | Type | Validation |
|-------|------|------------|
| Show Name | text | Required, 1–200 chars |
| Theatre Name | text | Required |
| Screen Number | number | >= 1 |
| Start Time | datetime-local | Must be in future |
| End Time | datetime-local | Must be > start_time |
| Price Per Seat (₹) | number | > 0 |
| Seat Layout | builder | Min 1 row, max 20 rows |

**Seat Layout Builder**:
- Dynamic row builder: Add/remove rows
- Per row: Row label (A-Z), seat count (1-30), seat type (Standard/Premium/Recliner)
- Live preview: Small seat grid preview below the builder
- Preset templates: "Standard (4 rows × 10 seats)", "Premium (6 rows × 12 seats)"
- Price modifiers: Standard = 1.0×, Premium = 1.5×, Recliner = 2.0×

**Behavior**:
- On submit: `POST /admin/shows` with form data
- On success: Navigate to `/admin` with success toast
- On validation error: Show field-level error messages inline

---

### 4.9 Admin — Show Analytics (`/admin/shows/:showId`)

**Purpose**: Per-show stats and seat management.

**Layout**:
- Show name + back link
- Stats row: Available / Locked / Booked seat counts + occupancy %
- Revenue: Total revenue from confirmed bookings
- Seat Override table: All seats with status. Admin can click to force-release a locked seat.
- "Force Release Seat" button per locked seat: `POST /admin/shows/:showId/seats/:seatId/override`
- "Cancel Show" button: `DELETE /admin/shows/:showId` with confirmation dialog

---

## 5. Component Library

### 5.1 Layout Components

| Component | Description |
|-----------|-------------|
| `AppShell` | Root layout: header + main content + footer |
| `PageHeader` | Title + optional subtitle + back button |
| `Card` | Surface container with optional border and shadow |
| `Modal` | Centered overlay with backdrop blur |
| `Toast` | Top-right notification: success/error/info/warning variants |
| `LoadingSkeleton` | Shimmer placeholder matching content layout |
| `EmptyState` | Icon + message + optional CTA for empty lists |
| `Badge` | Small inline label: status badges, seat type badges |
| `Spinner` | Loading indicator for buttons and content areas |

### 5.2 Seat Components

| Component | Description |
|-----------|-------------|
| `SeatGrid` | Renders all seats grouped by row |
| `SeatRow` | Single row with label + seats |
| `Seat` | Individual seat with all visual states |
| `SeatTooltip` | Hover tooltip showing seat details |
| `SeatLegend` | Color legend for seat states |
| `SeatLayoutBuilder` | Admin-only row builder for show creation |
| `SeatTypeBadge` | Small badge indicating Premium/Recliner |
| `ScreenIndicator` | Horizontal bar representing the screen |

### 5.3 Booking Components

| Component | Description |
|-----------|-------------|
| `LockTimer` | Countdown MM:SS with color states |
| `BookingCard` | Show card with booking details |
| `SeatSelectionPanel` | Selected seats list + total + CTA |
| `QueueStatusBanner` | Banner showing queue position and status |
| `TicketDisplay` | Styled ticket for confirmed bookings |

### 5.4 Form Components

| Component | Description |
|-----------|-------------|
| `Input` | Text input with label + error state |
| `NumberInput` | Numeric input with min/max |
| `DateTimeInput` | Datetime-local picker |
| `Select` | Dropdown select |
| `Button` | Primary (gold), Secondary (ghost), Danger variants |
| `Checkbox` | For multi-select seat selection (desktop) |
| `FormField` | Label + input + error message wrapper |

---

## 6. User Flows

### 6.1 Happy Path (Lock → Pay → Confirm)

```
1. User lands on / → sees show cards
2. Clicks "Select Seats" on a show
3. Lands on /shows/:showId → seat grid loads
4. Clicks 4 available seats → selection panel updates with total
5. Clicks "Lock Seats" → POST succeeds → navigates to /bookings/:bookingId
6. Sees 4:32 countdown timer + "Proceed to Payment" button
7. Clicks "Proceed to Payment" → /bookings/:bookingId/payment
8. Payment initiation call fires → form shown with ₹XXX amount
9. Enters card → clicks Pay → 2s spinner
10. Mock gateway succeeds → redirected to /bookings/:bookingId/confirmed
11. Ticket displayed → confetti animation
12. Clicks "View My Bookings" → /my-bookings → shows Confirmed badge
```

### 6.2 Conflict Flow (Seat Unavailable)

```
1. User selects seat A5
2. Another user already locked it (or it's booked)
3. User clicks "Lock Seats"
4. API returns 409 with conflicting seat IDs
5. UI highlights conflicting seats in red
6. Toast: "Seat A5 is already taken. Please choose different seats."
7. User deselects A5, picks A7 instead
8. Proceeds normally
```

### 6.3 Queue Flow (High Traffic)

```
1. User selects seats on a popular show
2. User clicks "Lock Seats"
3. Server returns: too many concurrent locks for this show → immediate queue entry
4. UI shows "You're in queue — position #3" banner
5. Poll GET /queue/:queueId/status every 1s
6. Position decrements: 3 → 2 → 1
7. Status becomes "Processing" → show spinner "Acquiring your seats..."
8. Status becomes "Locked" → navigate to booking page normally
9. OR status becomes "Conflict" → show modal: "Seats A1, A3 are no longer available. Please choose different seats." → return to seat selection
```

### 6.4 Lock Expiration Flow

```
1. User is on /bookings/:bookingId with 0:45 remaining
2. Polling every 10s detects status changed to "Expired"
3. Modal appears: "Your lock has expired. The seats have been released."
4. "Browse Shows" → redirect to /
5. Seats are now available for other users
```

### 6.5 Payment Failure Flow

```
1. User on payment page with 3:00 remaining
2. Clicks Pay → 2s loading
3. Mock gateway returns failure (or 20% random failure)
4. UI shows error banner: "Payment failed. Please try again."
5. Seats remain locked — user can retry or cancel
6. If user retries and succeeds → normal confirmation flow
7. If lock expires during retry → expiration modal
```

### 6.6 Lock Extension Flow

```
1. User on booking page with 2:30 remaining
2. UI shows "Extend Lock (+2 min)" button (not yet maxed)
3. User clicks → POST /extend-lock
4. Timer resets to 5:00 (2 min added, not cumulative with original TTL)
5. Button now shows "1 extension used" → one more available
6. User extends again → "2 extensions used" → button disabled + tooltip "Maximum extensions reached"
```

---

## 7. Admin Flows

### 7.1 Create Show

```
1. Admin navigates to /admin/shows/new
2. Fills form: show name, theatre, screen, times, price
3. Uses seat layout builder to define rows
4. Clicks "Create Show" → POST /admin/shows
5. Success toast: "Show created with X seats"
6. Redirects to /admin with new show in table
```

### 7.2 Force-Release Seat

```
1. Admin on /admin/shows/:showId
2. Sees seat A3 is "Locked" by user XXX
3. Clicks "Force Release" on seat A3
4. Confirmation modal: "Release seat A3? This will cancel the user's active lock."
5. Confirms → POST /admin/shows/:showId/seats/:seatId/override
6. Seat status changes to "Available" in UI
7. User whose lock was broken sees "Lock released by admin" modal on their booking page
```

### 7.3 Cancel Show

```
1. Admin on /admin/shows/:showId
2. Clicks "Cancel Show"
3. Confirmation modal with severity: "This will cancel all X active bookings and issue refunds."
4. Confirms → DELETE /admin/shows/:showId
5. All associated bookings marked cancelled, refunds initiated
6. Admin redirected to /admin with success toast
```

---

## 8. Error Handling & Edge Cases

### 8.1 Error Response Display

All API errors follow the backend's `ApiResponse` error envelope. The UI should:
- Parse `error.code` to determine the user-friendly message
- Parse `error.details` for contextual data (e.g., conflicting seat IDs)
- Display inline on the relevant form field, or as a toast for global errors

**Error Code → User Message Mapping**:

| Error Code | User Message |
|------------|-------------|
| `SEATS_UNAVAILABLE` | "One or more seats are no longer available. They have been highlighted." |
| `MAX_SEATS_EXCEEDED` | "You can select a maximum of 10 seats per booking." |
| `LOCK_EXPIRED` | "Your lock has expired. Please select seats again." |
| `LOCK_MAX_EXTENSIONS_REACHED` | "You've reached the maximum number of lock extensions." |
| `RATE_LIMIT_EXCEEDED` | "Too many requests. Please wait a moment." |
| `SHOW_NOT_FOUND` | "This show is no longer available." |
| `BOOKING_NOT_FOUND` | "Booking not found." |
| `VALIDATION_ERROR` | Show the specific validation message from `error.message` |

### 8.2 Network Error States

| Scenario | UI Behavior |
|----------|------------|
| API call fails (network error) | Toast: "Unable to connect. Check your internet connection." + Retry button |
| API timeout (>10s) | Toast: "Request timed out. Please try again." |
| Server returns 5xx | Toast: "Something went wrong on our end. Please try again in a moment." |
| Backend unreachable | Full-page error: "Backend server is not running. Please start it at localhost:8080." |

### 8.3 Empty & Loading States

| State | UI |
|-------|-----|
| Shows loading | Shimmer skeleton cards (3 cards) |
| Shows empty | EmptyState with movie icon + "No shows available" |
| Seat layout loading | Shimmer grid placeholder |
| Seat layout error | Error card with retry button |
| Booking loading | Booking card skeleton |
| Payment processing | Full-page spinner overlay: "Processing your payment..." |

### 8.4 Seat Map Polling Strategy

- On `/shows/:showId`: Poll `GET /shows/:showId/seats` every **5 seconds**
- On `/bookings/:bookingId`: Poll `GET /bookings/:bookingId` every **10 seconds** (to check lock expiry)
- During payment: Poll every **5 seconds** (lock expiry is critical here)
- Stop polling when user navigates away (clean up intervals on unmount)

### 8.5 localStorage Keys

| Key | Value | Purpose |
|-----|-------|---------|
| `bms_user_id` | UUID string | Identifies the user across sessions |
| `bms_lock_extensions` | `{bookingId: number}` | Tracks how many times user extended each lock |
| `bms_admin_token` | string | Admin auth token (if provided by admin user) |

---

## 9. Tech Stack & Architecture

### 9.1 Stack

- **Framework**: React 18+ with TypeScript, or Next.js 14+
- **Routing**: React Router v6 (or Next.js App Router)
- **HTTP Client**: Axios with interceptors for auth headers and error handling
- **State Management**: Zustand or React Context (for lock timer, queue state)
- **Styling**: CSS Modules or Tailwind CSS with custom design tokens (no SaaS template styles)
- **Date/Time**: date-fns (for formatting show times, countdown math)
- **Icons**: Lucide React (consistent, minimal icon set)
- **Build Tool**: Vite (recommended) or Next.js built-in

### 9.2 Project Structure (React)

```
frontend/
├── src/
│   ├── api/
│   │   ├── client.ts           # Axios instance with base URL + interceptors
│   │   ├── shows.ts            # Show API calls
│   │   ├── bookings.ts         # Booking API calls
│   │   ├── payments.ts         # Payment API calls
│   │   ├── queue.ts            # Queue API calls
│   │   └── admin.ts           # Admin API calls
│   ├── components/
│   │   ├── layout/             # AppShell, PageHeader, Modal, Toast
│   │   ├── seats/              # SeatGrid, Seat, SeatRow, SeatLegend, SeatTooltip
│   │   ├── booking/           # LockTimer, BookingCard, SeatSelectionPanel, TicketDisplay
│   │   ├── forms/             # Input, Button, Select, DateTimeInput, FormField
│   │   └── common/            # Badge, Spinner, EmptyState, LoadingSkeleton
│   ├── pages/
│   │   ├── Home.tsx
│   │   ├── SeatSelection.tsx
│   │   ├── BookingDetails.tsx
│   │   ├── Payment.tsx
│   │   ├── BookingConfirmed.tsx
│   │   ├── MyBookings.tsx
│   │   ├── admin/
│   │   │   ├── AdminDashboard.tsx
│   │   │   ├── CreateShow.tsx
│   │   │   └── ShowAnalytics.tsx
│   │   └── NotFound.tsx
│   ├── hooks/
│   │   ├── useApi.ts           # Generic fetch + loading + error state
│   │   ├── useLockTimer.ts     # Countdown timer hook
│   │   ├── useQueuePolling.ts  # Queue status polling hook
│   │   └── useUserId.ts        # Get/generate user UUID from localStorage
│   ├── stores/
│   │   └── bookingStore.ts     # Zustand store for active booking state
│   ├── types/
│   │   └── api.ts              # TypeScript types matching backend DTOs
│   ├── styles/
│   │   ├── tokens.css          # CSS custom properties (colors, spacing, typography)
│   │   └── globals.css
│   ├── utils/
│   │   ├── format.ts           # Format time, price, seat numbers
│   │   └── error.ts            # Error code → user message mapping
│   ├── App.tsx
│   └── main.tsx
├── public/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── .env
```

### 9.3 Environment Variables

```
VITE_API_BASE_URL=http://localhost:8080
VITE_ADMIN_TOKEN=admin-secret
```

### 9.4 API Client Setup

```typescript
// Axios instance with auth headers and error normalization
const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 10000,
});

// Request interceptor: inject X-User-Id header
api.interceptors.request.use((config) => {
  const userId = localStorage.getItem('bms_user_id');
  if (userId) config.headers['X-User-Id'] = userId;
  return config;
});

// Response interceptor: unwrap ApiResponse envelope, throw on !success
api.interceptors.response.use((response) => {
  if (response.data && 'success' in response.data) {
    if (response.data.success) {
      return { ...response, data: response.data.data };
    } else {
      throw new ApiError(response.data.error);
    }
  }
  return response;
});
```

---

## 10. Out of Scope

| Item | Reason |
|------|--------|
| Real payment gateway (Stripe/Razorpay) | Backend uses mock gateway; UI should match |
| User registration/login screens | User identity is UUID-based, stored in localStorage |
| JWT/session auth | Backend auth is header-based (X-User-Id, X-Admin-Token) |
| WebSocket real-time seat updates | Use HTTP polling every 5s for MVP |
| Multi-language / i18n | Single language (English) |
| Mobile native app | Web-only for MVP |
| Email/SMS notifications | Out of scope per PRD |
| Complex animations / 3D seat viewer | Simple 2D grid suffices for MVP |
| Dark/light theme toggle | Dark theme only |
| Seat drag-and-drop selection | Click-based selection only |
| Seat pre-selection for returning users | Out of scope |
