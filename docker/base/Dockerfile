# Dockerfile
FROM rust:1.82 as builder

# Install minimal system dependencies (no RocksDB C++ requirements)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --features production
RUN rm -rf src

# Build actual application
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Touch main.rs to ensure rebuild
RUN touch src/main.rs

# Build with production features
RUN cargo build --release --features production

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/petra /usr/local/bin/petra

# Create data directory
RUN mkdir -p /data

WORKDIR /app
EXPOSE 9090

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD petra --health-check || exit 1

ENTRYPOINT ["petra"]
CMD ["--config", "/app/config.yaml"]
