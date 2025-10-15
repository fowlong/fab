use axum::{extract::Path, extract::State, routing::{get, post}, Json, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod types;
mod pdf;
mod util;

use crate::types::{DocumentIR, PatchOperation};
use pdf::loader::PdfStore;

#[derive(Clone)]
struct AppState {
    store: PdfStore,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: PdfStore::default(),
    };

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn open_pdf(State(state): State<AppState>) -> Json<serde_json::Value> {
    let doc_id = state.store.create_placeholder();
    Json(serde_json::json!({ "docId": doc_id }))
}

async fn get_ir(State(state): State<AppState>, Path(doc_id): Path<String>) -> Json<DocumentIR> {
    Json(state.store.ir_for(&doc_id))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(_ops): Json<Vec<PatchOperation>>,
) -> Json<serde_json::Value> {
    // Placeholder implementation.
    let updated_pdf = state.store.updated_pdf(&doc_id);
    Json(serde_json::json!({ "ok": true, "updatedPdf": updated_pdf }))
}

async fn download_pdf(State(state): State<AppState>, Path(doc_id): Path<String>) -> Json<serde_json::Value> {
    let updated_pdf = state.store.updated_pdf(&doc_id);
    Json(serde_json::json!({ "data": updated_pdf }))
}
