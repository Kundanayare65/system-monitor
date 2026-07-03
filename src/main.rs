use axum::{routing::get, Router};
use std::net::SocketAddr;
use sysinfo::System;
use tower_http::cors::CorsLayer;

async fn metrics() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();

    serde_json::json!({
        "cpu_usage_percent": format!("{:.1}", cpu),
        "memory_used_mb": used_mem / 1024 / 1024,
        "memory_total_mb": total_mem / 1024 / 1024,
    })
    .to_string()
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();

    let app = Router::new()
        .route("/metrics", get(metrics))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Server running on http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}