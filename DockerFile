# ==========================================
# Stage 1: The Build Environment
# ==========================================
FROM rust:1.75 AS builder

WORKDIR /app

# Copy your source code and manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build your application for release optimization
RUN cargo build --release

# ==========================================
# Stage 2: The Tiny Runtime Environment
# ==========================================
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install SSL certificates (Crucial if your Rust backend makes HTTPS requests to external APIs)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
# ⚠️ REPLACE "your_app_name" with the actual 'name' found under [package] in your Cargo.toml
COPY --from=builder /app/target/release/echoes-backend ./backend-app

# Expose the port your Rust server listens on (e.g., 8080)
EXPOSE 8080

# Run the binary
CMD ["./backend-app"]