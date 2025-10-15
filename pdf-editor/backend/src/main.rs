use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};

mod pdf;
mod types;
mod util;

use crate::pdf::loader::DocumentStore;
use crate::types::{IntermediateRepresentation, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(DocumentStore::default());
    let state = AppState { store };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if let Some(file_name) = field.file_name() {
            println!("Received upload: {}", file_name);
        }
    }

    let doc_id = state
        .store
        .open_document_placeholder()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(OpenResponse { doc_id }))
}

async fn fetch_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<IntermediateRepresentation>, StatusCode> {
    let ir = state
        .store
        .document_ir(&doc_id)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(patch): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, StatusCode> {
    let response = state
        .store
        .apply_patch(&doc_id, patch)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<(axum::http::HeaderMap, Vec<u8>), StatusCode> {
    let data = state
        .store
        .pdf_bytes(&doc_id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/pdf".parse().unwrap(),
    );

    Ok((headers, data))
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: String,
}
