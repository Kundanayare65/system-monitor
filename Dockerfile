# Stage 1 — Build the Rust app
FROM rust:latest AS builder

# Create a working directory inside the container
WORKDIR /app

# Copy your project files into the container
COPY . .

# Compile the Rust app in release mode (faster, smaller binary)
RUN cargo build --release

# Stage 2 — Create a small final image with just the binary
FROM debian:bookworm-slim

WORKDIR /app

# Copy only the compiled binary from stage 1
COPY --from=builder /app/target/release/system-monitor .

# Tell Docker this app runs on port 3000
EXPOSE 3000

# Run the app when container starts
CMD ["./system-monitor"]