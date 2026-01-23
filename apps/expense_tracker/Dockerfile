# Multi-stage Dockerfile for Rust backend (expense_tracker)
# Builder stage
FROM rust:1.93-slim AS builder

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

# Install runtime deps and create non-root user
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libpq5 && \
#    cargo install diesel_cli --no-default-features --features postgres-bundled && \
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
