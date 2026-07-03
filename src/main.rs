use axum::{routing::get, Router};
use std::net::SocketAddr;
use sysinfo::System;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};

async fn metrics() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();

    serde_json::json!({
        "cpu_usage_percent": format!("{:.1}", cpu),
        "memory_used_mb": sys.used_memory() / 1024 / 1024,
        "memory_total_mb": sys.total_memory() / 1024 / 1024,
    })
    .to_string()
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap();

    let app = Router::new()
        .route("/metrics", get(metrics))
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("frontend/dist"));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    println!("🚀 Server running at http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}