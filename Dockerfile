# Build stage
FROM rust:1.79-slim-bullseye as builder

WORKDIR /usr/src/app
COPY . .

# Install OpenSSL development packages and pkg-config
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies first (for better caching)
RUN cargo build --release

# Final stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

# Copy the built binary from builder
COPY --from=builder /usr/src/app/target/release/temp ./app

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 -U app
USER app

# Expose the port the app runs on
EXPOSE 8000

CMD ["./app"]