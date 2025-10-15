use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Multipart, Path as AxumPath, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use serde::Serialize;
use thiserror::Error;
use tokio::signal;
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::extract::extract_ir;
use pdf::loader;
use pdf::patch::apply_patch;
use types::{DocumentIr, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<HashMap<String, StoredDocument>>>,
}

struct StoredDocument {
    bytes: Vec<u8>;
    ir: DocumentIr;
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("missing file field")] 
    MissingFile,
    #[error("invalid multipart payload")]
    InvalidMultipart(#[source] anyhow::Error),
    #[error("document not found")]
    NotFound,
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::MissingFile => StatusCode::BAD_REQUEST,
            ApiError::InvalidMultipart(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = match &self {
            ApiError::InvalidMultipart(source) | ApiError::Internal(source) => {
                format!("{}: {}", self.to_string(), source)
            }
            _ => self.to_string(),
        };
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::default();
    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch_route))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8787)).await?;
    println!("Server listening on http://{}/", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.map_err(|err| ApiError::InvalidMultipart(err.into()))? {
        if let Some(name) = field.name() {
            if name == "file" {
                let data = field.bytes().await.map_err(|err| ApiError::InvalidMultipart(err.into()))?;
                file_bytes = Some(data.to_vec());
                break;
            }
        }
    }
    let bytes = file_bytes.ok_or(ApiError::MissingFile)?;
    let loaded = loader::load(&bytes).map_err(ApiError::Internal)?;
    let ir = extract_ir(&loaded).map_err(ApiError::Internal)?;
    let doc_id = Uuid::new_v4().to_string();
    state.store.write().insert(
        doc_id.clone(),
        StoredDocument { bytes: loaded, ir: ir.clone() },
    );
    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    AxumPath(doc_id): AxumPath<String>,
) -> Result<Json<DocumentIr>, ApiError> {
    let guard = state.store.read();
    let doc = guard.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(doc.ir.clone()))
}

async fn apply_patch_route(
    State(state): State<AppState>,
    AxumPath(doc_id): AxumPath<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let response = apply_patch(&ops).map_err(ApiError::Internal)?;
    // In the placeholder implementation we do not modify stored bytes/IR yet.
    if !response.ok {
        return Ok(Json(response));
    }
    let mut guard = state.store.write();
    let doc = guard.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    // TODO: update doc.bytes and doc.ir once patching is implemented.
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    AxumPath(doc_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let guard = state.store.read();
    let doc = guard.get(&doc_id).ok_or(ApiError::NotFound)?;
    let body = doc.bytes.clone();
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf"), (header::CONTENT_DISPOSITION, "attachment; filename="document.pdf"")],
        body,
    )
        .into_response())
}
