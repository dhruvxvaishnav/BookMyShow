CREATE TABLE IF NOT EXISTS users (
    user_id     TEXT PRIMARY KEY,
    user_name   TEXT NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    role        TEXT NOT NULL DEFAULT 'user',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS movies (
    movie_id         TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    genre            TEXT NOT NULL,
    language         TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL,
    poster_url       TEXT,
    rating           REAL NOT NULL DEFAULT 0.0,
    description      TEXT NOT NULL DEFAULT '',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS venues (
    venue_id     TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    address      TEXT NOT NULL,
    city         TEXT NOT NULL,
    screen_count INTEGER NOT NULL DEFAULT 1,
    amenities    TEXT[] NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shows (
    show_id        TEXT PRIMARY KEY,
    show_name      TEXT NOT NULL,
    theatre_name   TEXT NOT NULL,
    screen_number  INTEGER NOT NULL DEFAULT 1,
    start_time     TIMESTAMPTZ NOT NULL,
    end_time       TIMESTAMPTZ NOT NULL,
    price_per_seat DOUBLE PRECISION NOT NULL,
    total_seats    INTEGER NOT NULL,
    movie_id       TEXT REFERENCES movies(movie_id) ON DELETE SET NULL,
    venue_id       TEXT REFERENCES venues(venue_id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS seats (
    seat_id            TEXT PRIMARY KEY,
    seat_number        TEXT NOT NULL,
    row_label          TEXT NOT NULL,
    seat_type          TEXT NOT NULL DEFAULT 'Standard',
    show_id            TEXT NOT NULL REFERENCES shows(show_id) ON DELETE CASCADE,
    status             TEXT NOT NULL DEFAULT 'Available',
    locked_by_user_id  TEXT,
    locked_at          TIMESTAMPTZ,
    lock_expires_at    TIMESTAMPTZ,
    lock_id            TEXT,
    booked_by_user_id  TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_seats_show_id     ON seats(show_id);
CREATE INDEX IF NOT EXISTS idx_seats_show_status ON seats(show_id, status);

CREATE TABLE IF NOT EXISTS seat_locks (
    lock_id        TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL,
    show_id        TEXT NOT NULL REFERENCES shows(show_id) ON DELETE CASCADE,
    seat_ids       TEXT[] NOT NULL DEFAULT '{}',
    status         TEXT NOT NULL DEFAULT 'Active',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL,
    extended_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_seat_locks_show_id ON seat_locks(show_id);
CREATE INDEX IF NOT EXISTS idx_seat_locks_status  ON seat_locks(status);

CREATE TABLE IF NOT EXISTS bookings (
    booking_id     TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL,
    show_id        TEXT NOT NULL REFERENCES shows(show_id) ON DELETE CASCADE,
    seat_ids       TEXT[] NOT NULL DEFAULT '{}',
    status         TEXT NOT NULL DEFAULT 'Pending',
    payment_id     TEXT,
    total_amount   DOUBLE PRECISION NOT NULL,
    lock_id        TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL,
    confirmed_at   TIMESTAMPTZ,
    cancelled_at   TIMESTAMPTZ,
    seats_snapshot JSONB
);

CREATE INDEX IF NOT EXISTS idx_bookings_user_id    ON bookings(user_id);
CREATE INDEX IF NOT EXISTS idx_bookings_show_id    ON bookings(show_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status     ON bookings(status);
CREATE INDEX IF NOT EXISTS idx_bookings_payment_id ON bookings(payment_id);

CREATE TABLE IF NOT EXISTS payments (
    payment_id         TEXT PRIMARY KEY,
    payment_intent_id  TEXT NOT NULL UNIQUE,
    booking_id         TEXT NOT NULL REFERENCES bookings(booking_id) ON DELETE CASCADE,
    user_id            TEXT NOT NULL,
    amount             DOUBLE PRECISION NOT NULL,
    currency           TEXT NOT NULL DEFAULT 'INR',
    status             TEXT NOT NULL DEFAULT 'Pending',
    gateway_name       TEXT NOT NULL,
    gateway_response   TEXT,
    idempotency_key    TEXT UNIQUE,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at       TIMESTAMPTZ,
    failed_at          TIMESTAMPTZ,
    refunded_at        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_payments_booking_id ON payments(booking_id);
CREATE INDEX IF NOT EXISTS idx_payments_user_id    ON payments(user_id);
CREATE INDEX IF NOT EXISTS idx_payments_intent_id  ON payments(payment_intent_id);

CREATE TABLE IF NOT EXISTS queue_entries (
    queue_id           TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL,
    show_id            TEXT NOT NULL,
    requested_seat_ids TEXT[] NOT NULL DEFAULT '{}',
    status             TEXT NOT NULL DEFAULT 'Waiting',
    position           INTEGER NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at       TIMESTAMPTZ,
    conflict_seats     TEXT[],
    booking_id         TEXT,
    lock_id            TEXT
);

CREATE INDEX IF NOT EXISTS idx_queue_show_id ON queue_entries(show_id);
CREATE INDEX IF NOT EXISTS idx_queue_status  ON queue_entries(status);

CREATE TABLE IF NOT EXISTS compensation_logs (
    compensation_id TEXT PRIMARY KEY,
    booking_id      TEXT NOT NULL,
    show_id         TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    confirmed_seats TEXT[] NOT NULL DEFAULT '{}',
    failed_seats    TEXT[] NOT NULL DEFAULT '{}',
    total_amount    DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    failed_amount   DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    event_type      TEXT NOT NULL DEFAULT 'partial_booking',
    actor_id        TEXT,
    status_from     TEXT,
    status_to       TEXT,
    message         TEXT,
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compensation_booking_id ON compensation_logs(booking_id);
CREATE INDEX IF NOT EXISTS idx_compensation_user_id    ON compensation_logs(user_id);
