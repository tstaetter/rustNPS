# Stage 1: Build the application
FROM docker.io/rust:1-alpine as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-json recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Stage 2: Run the application
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (like OpenSSL for MongoDB/HTTPS)
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/rust_nps /app/rust_nps

# Expose the port the app runs on
EXPOSE 8000

# Run the binary
CMD ["./rust_nps"]
