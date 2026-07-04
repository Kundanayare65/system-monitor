FROM rust:1.77 as builder

WORKDIR /app

# Install openssl-dev for sysinfo
RUN apt-get update && apt-get install -y libssl-dev pkg-config

COPY . .

# Build the Rust backend
RUN cargo build --release

# Build the React frontend
WORKDIR /app/frontend
RUN npm install
RUN npm run build

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install openssl-dev for sysinfo (runtime dependency)
RUN apt-get update && apt-get install -y openssl

COPY --from=builder /app/target/release/system-monitor .
COPY --from=builder /app/frontend/dist ./frontend/dist

EXPOSE $PORT

CMD ["./system-monitor"]
