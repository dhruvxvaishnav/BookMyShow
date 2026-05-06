FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first so cargo can cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates/

# Build release binary
RUN cargo build --release --bin api

# ── Runtime image ──────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api /app/api
COPY config.toml /app/config.toml

ENV RUST_LOG=info
ENV APP_PORT=8080

EXPOSE 8080

CMD ["/app/api"]
