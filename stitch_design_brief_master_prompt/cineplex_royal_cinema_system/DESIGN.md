---
name: Cineplex Royal Cinema System
colors:
  surface: '#19120a'
  surface-dim: '#19120a'
  surface-bright: '#40382e'
  surface-container-lowest: '#130d06'
  surface-container-low: '#211a12'
  surface-container: '#251e16'
  surface-container-high: '#302920'
  surface-container-highest: '#3c332a'
  on-surface: '#eee0d2'
  on-surface-variant: '#d7c3ae'
  inverse-surface: '#eee0d2'
  inverse-on-surface: '#372f26'
  outline: '#9f8e7a'
  outline-variant: '#524534'
  surface-tint: '#ffb955'
  primary: '#ffc880'
  on-primary: '#452b00'
  primary-container: '#f5a623'
  on-primary-container: '#644000'
  inverse-primary: '#835500'
  secondary: '#ffb3ae'
  on-secondary: '#630d10'
  secondary-container: '#862726'
  on-secondary-container: '#ff9e98'
  tertiary: '#9bd9ff'
  on-tertiary: '#00344a'
  tertiary-container: '#3ac2ff'
  on-tertiary-container: '#004d6a'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#ffddb4'
  primary-fixed-dim: '#ffb955'
  on-primary-fixed: '#291800'
  on-primary-fixed-variant: '#633f00'
  secondary-fixed: '#ffdad7'
  secondary-fixed-dim: '#ffb3ae'
  on-secondary-fixed: '#410004'
  on-secondary-fixed-variant: '#832524'
  tertiary-fixed: '#c4e7ff'
  tertiary-fixed-dim: '#7cd0ff'
  on-tertiary-fixed: '#001e2c'
  on-tertiary-fixed-variant: '#004c69'
  background: '#19120a'
  on-background: '#eee0d2'
  surface-variant: '#3c332a'
  void-black: '#0D0D0F'
  deep-surface: '#161619'
  elevated-surface: '#1E1E23'
  cinema-black: '#0A0608'
  royal-crimson: '#7B1F1F'
  antique-gold: '#F5A623'
  gold-foil: '#D4A843'
  bright-gold: '#F5C842'
  ornament-bronze: '#8B7355'
  parchment-text: '#F5F5F7'
  stone-text: '#9CA3AF'
  muted-text: '#8B96A0'
  seat-green: '#22C55E'
  seat-gold: '#F5A623'
  seat-red: '#EF4444'
  seat-gray: '#6B7280'
  premium-purple: '#A855F7'
  recliner-teal: '#06B6D4'
  alert-red: '#EF4444'
  success-green: '#22C55E'
typography:
  display-hero:
    fontFamily: Playfair Display
    fontSize: 48px
    fontWeight: '700'
    lineHeight: '1.1'
    letterSpacing: -0.02em
  headline-marquee:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: 0.2em
  body-main:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.6'
    letterSpacing: '0'
  data-mono:
    fontFamily: JetBrains Mono
    fontSize: 14px
    fontWeight: '500'
    lineHeight: '1'
    letterSpacing: -0.01em
  label-small:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '500'
    lineHeight: '1.4'
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 8px
  container-padding: 24px
  gutter: 16px
  section-gap: 48px
  hero-height: 70vh
---

# Cineplex — Visual Design Brief
### For Stitch

---

## The One-Line Brief

Design a **premium movie ticket booking app** that feels like walking into a grand 1920s Royal Cinema — dark velvet walls, warm gold lighting, crimson curtains — reimagined on a modern OLED screen.

---

## Mood & Atmosphere

Imagine the lobby of an old, opulent picture palace. Dark. Hushed. A little dramatic. Gold light spills from ornate sconces. The carpet is deep crimson. There is a sense of occasion — you are not buying a product, you are claiming your seat at an event.

**Reference points:** The Grand Budapest Hotel colour palette. Art Deco poster typography. A Criterion Collection Blu-ray menu. The opening crawl of a prestige film.

**What this is NOT:** A SaaS productivity app. A food delivery app. A streaming service. There are no whites, no sky blues, no flat illustrations, no emoji, no rounded bubbly pill buttons.

---

## Colour Palette

| Swatch Name | Hex | Where It Lives |
|---|---|---|
| Void Black | `#0D0D0F` | Every page background |
| Deep Surface | `#161619` | Cards, navigation bar |
| Elevated Surface | `#1E1E23` | Modals, dropdowns |
| Cinema Black | `#0A0608` | The seat selection page only |
| Royal Crimson | `#7B1F1F` | Hero sections, featured accents, curtain effect |
| Antique Gold | `#F5A623` | Primary CTA buttons, active states, prices |
| Gold Foil | `#D4A843` | Large display text, headings on dark |
| Bright Gold | `#F5C842` | Hyperlinks, highlights |
| Ornament Bronze | `#8B7355` | Decorative dividers, Art Deco lines |
| Parchment Text | `#F5F5F7` | Primary body text |
| Stone Text | `#9CA3AF` | Secondary / supporting text |
| Muted Text | `#8B96A0` | Labels, captions, placeholders |
| Seat Green | `#22C55E` | Available seats |
| Seat Gold | `#F5A623` | Seats you have selected |
| Seat Red | `#EF4444` | Seats locked by other users |
| Seat Gray | `#6B7280` | Already booked seats |
| Premium Purple | `#A855F7` | Premium seat type indicator |
| Recliner Teal | `#06B6D4` | Recliner seat type indicator |
| Alert Red | `#EF4444` | Errors, warnings, cancellations |
| Success Green | `#22C55E` | Confirmed states |

**Rule:** The background is always dark. Text is always light. The only warm light comes from gold. The only drama comes from crimson.

---

## Typography

| Use | Font | Style |
|---|---|---|
| Movie titles, hero headings | **Playfair Display** | Bold, serif, theatrical |
| All UI — buttons, labels, nav, body | **Inter** | Clean, modern, neutral |
| Seat numbers, booking IDs, times, countdowns | **JetBrains Mono** | Monospace, precise, data-like |

**Typography rules:**
- Section labels (e.g., "NOW SHOWING") are ALL CAPS with wide letter spacing (like a marquee sign)
- Movie titles always use Playfair Display — never Inter
- Prices and booking codes always use JetBrains Mono
- Nothing uses a font size smaller than 11px

---

## Decorative Language (Art Deco Details)

These small details define the Royal Cinema personality:

1. **Ornamental divider:** A horizontal line that fades from transparent → bronze → gold → bronze → transparent. Used between major sections.
2. **Corner brackets:** On movie poster cards, four thin gold lines form decorative corners (like photo corners). They appear on hover.
3. **Gold accent bar:** The top edge of modals and login cards has a 4px bar with a crimson-to-gold-to-crimson gradient.
4. **Marquee venue names:** Theatre/venue names are rendered in uppercase with wide letter spacing in Gold Foil colour.
5. **Screen glow:** On the seat map page, the "SCREEN" indicator at the top glows softly in gold.

---

## Screen 1 — Home Page

**Above the fold:** A full-viewport-height hero. The featured movie's poster is blurred and stretched edge-to-edge as a background (very dark, very blurry — it's atmosphere, not image). Over it, on the right side, the poster is sharp and prominent. On the left, the movie title in Playfair Display — huge, gold — with a genre badge in crimson, a star rating, and a large "Book Now" button in Antique Gold.

**Below the fold:**
- Section: "NOW SHOWING" — uppercase marquee heading, ornamental divider below it, then a horizontal scroll row of movie cards.
- Section: "COMING SOON" — same treatment.

**Movie card:**
- Portrait format (roughly a phone screen shape)
- Movie poster fills the card
- Title in Playfair Display below the poster
- Star rating with a small gold star
- Genre as a small crimson badge in the top corner
- On hover: card lifts slightly, gold corner brackets appear, subtle gold glow

**Navigation bar:**
- Dark surface (`#161619`), slim gold bottom border
- Left: Logo — "🎬 CINEPLEX" in Playfair Display, gold
- Centre: Home, Movies, My Bookings links
- Right: City selector (a dropdown showing the user's chosen city) + Login button

---

## Screen 2 — Login / Register

**Centred card** on a near-black background. The card is `#161619` with a `box-shadow` in deep black. The top of the card has the 4px crimson-to-gold gradient accent bar.

- App logo centred at top
- Heading "Sign in to your account" in Playfair Display
- Email field + Password field (dark input backgrounds, gold border on focus)
- Full-width Antique Gold "Sign In" button
- "Create account →" link below
- Register page is identical but with Name + Confirm Password fields

---

## Screen 3 — Movie Listing

**Filter bar** sticky below the navbar: Genre dropdown + Language dropdown. Both on a dark surface bar with a gold bottom border.

**Grid of movie cards** (3 columns on desktop, 2 on tablet, 1 on mobile). Same card design as the home page but taller, showing a 2-line movie description below the title. Duration shown in JetBrains Mono in muted text.

**Empty state** when filters match nothing: A film reel icon, "No movies match your filters", a small "Clear Filters" button.

---

## Screen 4 — Movie Detail Page

**Hero section** (`70%` viewport height): Blurred dark poster backdrop. Sharp poster on the left (around 280px wide). On the right: movie title in Playfair Display 3rem gold, then rating, genre, language, duration, and full description. A large gold "Book Tickets" button that scrolls the page down.

**Showtimes section:**
- City pills — small filter buttons showing available cities. Selected city is gold-filled.
- For each theatre in that city:
  - Theatre name in marquee style (uppercase, letter-spaced, Gold Foil colour)
  - Theatre address in muted text
  - Amenity badges (e.g., "Dolby Atmos", "4K") as small flat badges in `#1E1E23`
  - A row of showtime pill buttons showing the time (e.g., "2:30 PM"). Green-tinted if seats are available, dark gray if sold out.

---

## Screen 5 — Seat Selection (The Heart of the App)

**This screen is the cinema auditorium. It must feel immersive.**

Background is Cinema Black (`#0A0608`) — darker than everywhere else.

**Layout top to bottom:**
1. Header bar (show name, movie, time)
2. Conditional queue banner (see below)
3. "SCREEN" curve — a glowing gold curved line at the top of the seating area, labelled "SCREEN" in tiny monospace uppercase below it
4. The seat grid
5. Seat legend
6. Sticky bottom summary bar

**The seat grid:**
- Rows labelled A, B, C… on both sides in JetBrains Mono gold
- Each seat is a small rounded-top rectangle (like a cinema seat viewed from above)
- Colour tells you everything at a glance:
  - Pale green with green border = available
  - Solid gold with glow = you have selected this seat
  - Pale red, slightly transparent = locked by someone else (cannot click)
  - Dark gray, very transparent = permanently booked (cannot click)
- Seat type is indicated by **border colour**: no special border = Standard, purple border = Premium, teal border = Recliner
- Hovering an available seat shows a small tooltip: seat label + type + price

**Seat legend:**
Horizontal row below the grid with 6 small labelled swatches: Available, Selected, Locked (Others), Booked, Premium, Recliner.

**Queue banner** (only shown when the user is waiting in line):
A full-width banner above the seat grid. Dark elevated background, crimson border. Shows a spinning clock icon, "You are in the queue — Position 12" and a subtle pulsing animation. This automatically disappears when the user reaches the front of the queue. **This state is real — the backend has a queue system for high-demand shows.**

**Sticky bottom bar:**
- Left: List of selected seat labels (e.g., "C4, C5, D3") in JetBrains Mono
- Centre: Seat type breakdown (e.g., "2 Premium + 1 Standard")
- Right: Total price in bold + a large "Lock Seats" Antique Gold button
- Button is greyed and disabled when no seats are selected

---

## Screen 6 — Payment / Checkout

**This screen has a ticking clock. The user has 5 minutes to pay before their seats are released.**

**The countdown timer** is the most prominent element on the page. Full-width banner at the very top. It shows `MM:SS` in JetBrains Mono, large. Normally gold-bordered. **When under 60 seconds it turns red and pulses urgently.** This is not decorative — if it hits zero, the seats are gone and the user is sent back.

**Two-column layout below the timer:**

*Left column — Order Summary:*
- Movie title in Playfair Display
- Show date and time (formatted, e.g., "Friday, 9 May • 7:30 PM")
- Venue name + screen number
- Table of booked seats with their type and price per seat
- Subtotal + Convenience fee + **Total** (bold, large, gold)

*Right column — Payment Form:*
- Heading "Complete Payment"
- Card input field (dark background, gold border, white text — matches the page theme)
- Large "Pay ₹{total}" gold button
- A small red "Cancel booking" text link at the bottom

---

## Screen 7 — E-Ticket Confirmation

**The emotional high point. This must feel like receiving something beautiful.**

**Top:** Confetti burst animation in gold and crimson. A large green checkmark circle scales in.

**The Ticket:**
A stylised digital ticket stub that mimics a physical cinema ticket:
- Left accent stripe: a narrow vertical bar with a crimson-to-gold gradient
- Left portion (main ticket): Movie title in Playfair Display, show time, venue + screen, booked seat labels as small gold chips (e.g., "C4", "C5"), total amount
- A dashed vertical line suggesting a perforated tear edge
- Right stub: A QR code in gold on dark background, booking reference number in tiny JetBrains Mono below it

The entire ticket has a subtle gold border and a dark shadow — it should look like something you'd want to screenshot.

**Below the ticket:**
- "Download / Print Ticket" button (with a printer icon)
- "View My Bookings" outline button

---

## Screen 8 — My Bookings

**Two tabs:** "Upcoming" and "Past"

**Booking card (horizontal):**
- Left: small movie poster thumbnail
- Middle: movie title in Playfair Display, show date/time in JetBrains Mono, venue name, seat labels
- Right: total amount bold + a status badge
- Status badge colours: Success = green, Awaiting Payment = amber, Cancelled = red, Expired = muted gray, Partially Confirmed = gold

**Pagination** at the bottom: ← 1 of 5 →

---

## Admin Portal (Screens 9–14)

**Shift in tone:** The admin portal uses the same dark colour palette but is denser, more clinical. Data tables replace movie posters. The cinematic drama is dialled down. Still dark, still gold accents — but the priority is legibility and information density, not atmosphere.

**Sidebar navigation** (240px wide, fixed):
- Logo + "ADMIN" label
- Nav links with Lucide icons: Dashboard, Shows, Bookings, Audit Logs
- Active link has a 3px gold left border and gold text
- Current user email + Logout at the bottom

---

### Admin Screen 9 — Dashboard

**4 KPI cards in a row:**
- Total Revenue (gold, money icon)
- Active Bookings (green, checkmark icon)
- Seats Currently Locked (amber, lock icon)
- Total Shows (blue, film icon)

Each card: dark surface, large number in Inter Bold, label in muted text, coloured icon top-right.

**Below:** A data table of all shows with columns: Show Name, Occupancy (shown as a thin progress bar), Revenue, Cancelled. Clicking a row goes to the Live Show Monitor.

---

### Admin Screen 10 — Admin Login

Same as user login but with a shield/admin icon, heading "Admin Portal".

---

### Admin Screen 11 — Create Show + Seat Layout Builder

**Two-panel form:**

*Left panel — Show Details:*
Form fields: Show Name, Theatre Name, Screen Number, Date + Start Time, Date + End Time, Base Price Per Seat (₹).

*Right panel — Seat Layout Builder:*
This is the most complex admin UI. The admin builds the theatre layout row by row.

- At the top: an "Add Row" mini-form: Row Label (e.g., "A"), Number of Seats (e.g., 20), Seat Type (Standard / Premium / Recliner). "Add Row" button.
- Below: each added row appears in a list — row label in JetBrains Mono gold, a strip of tiny coloured seat squares matching the type (green for Standard, purple for Premium, teal for Recliner), seat count, a delete button.
- At the very bottom of the panel: a **live mini-preview** of the seat layout being built. Shows all rows as tiny squares arranged like a real theatre, with the SCREEN indicator at top. This updates as rows are added or removed.

"Create Show" button at the bottom of the full form.

---

### Admin Screen 12 — Live Show Monitor

**Purpose:** Watch a specific show's seats in real time. See which are locked vs booked.

- Header: show details, availability stats (X booked / Y locked / Z available)
- The full seat grid (same visual as the user seat map) but in read-only mode
- Locked seats (red) have a small "Release" button that appears on hover — clicking it shows a confirmation modal before taking action
- A "Refresh" button to reload seat data

---

### Admin Screen 13 — All Bookings

**Data table:**
Booking ID (monospace, truncated), User, Show, Seats, Amount, Status badge, Date.

Filter bar at top: Status dropdown, search field.
Clicking a row opens a modal with the full booking detail.

---

### Admin Screen 14 — Audit Logs

**Purpose:** Shows partial failures and system events for support staff.

**Dense data table:**
- Audit ID (mono), Booking ID, Event Type (coloured badge), Status change arrow (`Pending → Success`), Confirmed seats (green pills), Failed seats (red pills), Failed amount (red text if > 0), Timestamp (full date + time)
- Rows with failed seats have a subtle red background tint — they visually stand out

---

## What the Design Must NOT Include

These features do not exist in the backend and must not appear in any screen:

- Social login (Google, Facebook, Apple) — only email/password
- Wishlist or "Save for Later"
- Movie reviews or user ratings (ratings shown are editorial, not user-submitted)
- Seat map zoom controls — the grid is fixed
- Promo codes or discount fields
- Loyalty points or rewards system
- "Recommended for you" AI suggestions
- Live chat or chatbot widget
- Push notification permission prompts
- Dark/light mode toggle — the app is always dark

---

## Interaction Moments Worth Animating

These are the moments that make the experience feel alive:

1. **Movie card hover** — card lifts, gold corner brackets slide in from the corners
2. **Seat selection** — seat bounces slightly when clicked, transitions from green to gold
3. **Lock Seats button** — brief loading state with a spinning icon, then page transition
4. **Queue banner** — clock icon spins, position number updates with a subtle fade
5. **Countdown timer going red** — smooth colour transition from gold to red, then pulse
6. **Ticket reveal on confirmation** — ticket slides up from below, confetti falls

---

## Summary Personality Words

**Dark. Theatrical. Gold. Deliberate. Premium. Cinematic.**

Every screen should feel like it was designed by someone who loves cinema. Not by someone who loves enterprise software.
