# System Monitor

A lightweight system monitoring REST API built with Rust, containerized with Docker.

## What it does
Exposes live CPU and memory stats as a JSON API endpoint.

## Tech Stack
- Rust (Axum, Tokio, Serde, sysinfo)
- Docker
- GitHub Actions (CI/CD)

## Run locally with Docker
docker run -p 3000:3000 kundanayare65/system-monitor

## API
GET http://localhost:3000/metrics

## Response
{
  "cpu_usage_percent": "18.3",
  "memory_used_mb": 6544,
  "memory_total_mb": 8192
}