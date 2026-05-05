# Base image for building
FROM rust:1.80-slim-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev protobuf-compiler build-essential && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates/

# Build release
RUN cargo build --release --bin api

# Runtime image
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies (OpenSSL usually required)
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/api /app/api
COPY backend/config.toml /app/config.toml

# Default environment variables
ENV RUST_LOG=info
ENV APP_PORT=8080

EXPOSE 8080

# Run
CMD ["/app/api"]
