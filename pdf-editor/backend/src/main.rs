use axum::{routing::get, routing::post, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/open", post(not_implemented))
        .route("/api/ir/:doc_id", get(not_implemented))
        .route("/api/patch/:doc_id", post(not_implemented))
        .route("/api/pdf/:doc_id", get(not_implemented));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    println!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn not_implemented() -> axum::http::StatusCode {
    axum::http::StatusCode::NOT_IMPLEMENTED
}
