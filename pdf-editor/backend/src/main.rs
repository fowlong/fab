use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use crate::types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Default)]
struct AppState {
    _placeholder: (),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/api/open", post(open))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state)
        .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any));

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    info!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn open(State(_state): State<Arc<AppState>>) -> Json<OpenResponse> {
    let doc_id = Uuid::new_v4().to_string();
    Json(OpenResponse { doc_id })
}

async fn get_ir(State(_state): State<Arc<AppState>>) -> Json<DocumentIR> {
    Json(DocumentIR::empty())
}

async fn apply_patch(
    State(_state): State<Arc<AppState>>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Json<PatchResponse> {
    Json(PatchResponse::acknowledged(ops.len()))
}

async fn download_pdf(State(_state): State<Arc<AppState>>) -> axum::response::Response {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../e2e/sample.pdf");

    let bytes = fs::read(path).await.unwrap_or_default();
    axum::response::Response::builder()
        .header("content-type", "application/pdf")
        .body(axum::body::Body::from(bytes))
        .unwrap()
}
