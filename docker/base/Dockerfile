# Dockerfile
FROM rust:1.82 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with all features
RUN cargo build --release --features advanced-storage

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/petra /app/petra

WORKDIR /app
EXPOSE 9090

CMD ["/app/petra"]
