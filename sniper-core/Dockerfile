# Multi-stage Dockerfile for sniper-core
# Optimized for Contabo VDS production deployment

# ===== BUILD STAGE =====
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first (for better caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --profile contabo
RUN rm src/main.rs

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release --profile contabo

# ===== RUNTIME STAGE =====
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r sniper && useradd -r -g sniper sniper

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/sniper-core /app/sniper-core

# Copy configuration files
COPY .env.example /app/.env.example

# Create necessary directories
RUN mkdir -p /app/logs /app/data

# Set ownership
RUN chown -R sniper:sniper /app

# Switch to non-root user
USER sniper

# Expose port
EXPOSE 8003

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8003/health || exit 1

# Environment variables for Contabo VDS optimization
ENV RUST_LOG=sniper_core=info
ENV TOKIO_WORKER_THREADS=4
ENV MALLOC_CONF=background_thread:true,metadata_thp:auto

# Set resource limits (will be overridden by docker-compose)
ENV SNIPER_MAX_CONNECTIONS=1000
ENV SNIPER_REQUEST_TIMEOUT_MS=5000

# Default to paper trading for safety
ENV SNIPER_TRADING_MODE=paper

# Start the application
CMD ["./sniper-core"]
