# Multi-stage build for SniperBot
# Stage 1: Build the Rust application
FROM rust:1.75-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY configs ./configs

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false sniperbot

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/configs /app/python_executables \
    && chown -R sniperbot:sniperbot /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/sniper-bot /app/sniper-bot

# Copy configuration files
COPY --from=builder /app/configs /app/configs

# Set ownership
RUN chown -R sniperbot:sniperbot /app

# Switch to app user
USER sniperbot

# Set working directory
WORKDIR /app

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./sniper-bot", "--config", "configs/bot.toml"]
