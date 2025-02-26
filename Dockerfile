# Compile the Rust binary
FROM rust:1.83.0-slim-bullseye AS builder

WORKDIR /usr/src/app
COPY . .

# Update and install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Build the application in release mode
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bullseye-slim

# Install OpenSSL and CA certificates for HTTPS requests
RUN apt-get update && \
    apt-get install -y ca-certificates  && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/discord-proxy-bot /app/discord-proxy-bot

# Run the binary

ENTRYPOINT ["/app/discord-proxy-bot"]