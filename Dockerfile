# Multi-stage build for optimized production image
FROM rust:1.75 AS builder

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/

# Create dummy source files to cache dependencies
RUN mkdir -p crates/api/src crates/core/src crates/database/src crates/auth/src crates/monitoring/src
RUN echo "fn main() {}" > crates/api/src/main.rs
RUN echo "// dummy" > crates/core/src/lib.rs
RUN echo "// dummy" > crates/database/src/lib.rs
RUN echo "// dummy" > crates/auth/src/lib.rs
RUN echo "// dummy" > crates/monitoring/src/lib.rs

# Build dependencies
RUN cargo build --release
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/
COPY migrations/ migrations/

# Build the application
RUN cargo build --release --bin api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/migrations /app/migrations
COPY config/ config/

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser
RUN chown -R appuser:appuser /app
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080 9090

CMD ["api"]
