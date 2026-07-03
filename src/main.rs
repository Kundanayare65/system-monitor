use axum::{routing::get, Router};
use sysinfo::System;
use std::net::SocketAddr;

async fn metrics() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    // global_cpu_info() returns a Cpu object, .cpu_usage() gets the number from it
    let cpu = sys.global_cpu_info().cpu_usage();

    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();

    serde_json::json!({
        // cpu is now a plain f32 number so format works fine
        "cpu_usage_percent": format!("{:.1}", cpu),
        "memory_used_mb": used_mem / 1024 / 1024,
        "memory_total_mb": total_mem / 1024 / 1024,
    }).to_string()
}

#[tokio::main]
async fn main() {
    // Read PORT from environment — Railway sets this automatically
    // If no PORT set, default to 3000 for local development
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();

    let app = Router::new().route("/metrics", get(metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on port {}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}