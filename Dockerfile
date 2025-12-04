# Multi-stage build for optimized image size

# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
	apt-get install -y pkg-config libssl-dev && \
	rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (cached layer)
RUN mkdir src && \
	echo "fn main() {}" > src/main.rs && \
	cargo build --release && \
	rm -rf src

# Copy source code
COPY src ./src

# Build application (touch main.rs to force rebuild)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
	apt-get install -y ca-certificates && \
	rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/currency-converter-api .

# Expose port
EXPOSE 3000

# Run the application
CMD ["./currency-converter-api"]
