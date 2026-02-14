# Multi-stage Dockerfile for Rust backend (expense_tracker)
# This Dockerfile should be built from the project root to include workspace dependencies and config.
# Usage: docker build -f apps/expense_tracker/Dockerfile . -t api-test
# Builder stage
FROM rust:slim-bookworm AS builder

# Install build dependencies for crates like openssl
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates curl libpq-dev && \
    cargo install diesel_cli --no-default-features --features postgres && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace (simplest and most reliable for workspaces)
COPY . .

# Build the backend in release mode
# Use --bin expense_tracker to build only the API binary
RUN cargo build --release --bin expense_tracker --target-dir ./target

# Runtime stage
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source=https://github.com/Ciel1996/expense-tracker-backend
LABEL org.opencontainers.image.description="The backend of Expense Tracker."
LABEL org.opencontainers.image.licenses=GPL-3.0-or-later

# Install runtime deps and create non-root user
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libpq5 && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 10001 appuser

WORKDIR /app

# Copy binary and default config
COPY --from=builder /app/target/release/expense_tracker /app/expense_tracker
COPY config /app/config

# Ensure binary is executable (usually preserved, but explicit for safety)
RUN chmod +x /app/expense_tracker

ENV RUST_LOG=info
EXPOSE 3001

USER appuser

# The app reads config from config/settings.toml by default
CMD ["/app/expense_tracker"]
