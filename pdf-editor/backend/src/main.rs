use std::{net::SocketAddr, sync::Arc};

use axum::extract::Multipart;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::extract;
use pdf::loader::{DocumentStore, StoredDocument};
use pdf::patch;
use pdf::write;
use types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        store: Arc::new(DocumentStore::new()),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    println!("Listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: String,
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, AppError> {
    let mut pdf_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(AppError::from_anyhow)?
    {
        if let Some(name) = field.name() {
            if name == "file" {
                pdf_bytes = Some(field.bytes().await.map_err(AppError::from_anyhow)?.to_vec());
                break;
            }
        }
    }

    let bytes =
        pdf_bytes.ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "missing file field"))?;
    let ir = extract::extract_ir(&bytes).map_err(AppError::from_anyhow)?;
    let doc = StoredDocument::new(bytes, ir);
    let doc_id = state.store.insert(doc);
    Ok(Json(OpenResponse {
        doc_id: doc_id.to_string(),
    }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, AppError> {
    let uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "invalid doc id"))?;
    let stored = state
        .store
        .get(&uuid)
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "document not found"))?;
    Ok(Json(stored.ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, AppError> {
    let uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "invalid doc id"))?;
    let stored = state
        .store
        .get(&uuid)
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "document not found"))?;

    let mut outcome = patch::apply_patch(&stored, &ops).map_err(AppError::from_anyhow)?;
    let updated_bytes =
        write::incremental_update(outcome.updated_bytes).map_err(AppError::from_anyhow)?;
    if outcome.response.updated_pdf.is_none() {
        let data_url = format!(
            "data:application/pdf;base64,{}",
            BASE64.encode(&updated_bytes)
        );
        outcome.response.updated_pdf = Some(data_url);
    }
    let mut updated_doc = stored;
    updated_doc.current_bytes = updated_bytes.clone();
    state.store.update(&uuid, updated_doc);
    Ok(Json(outcome.response))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, AppError> {
    let uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "invalid doc id"))?;
    let stored = state
        .store
        .get(&uuid)
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "document not found"))?;
    let mut response = Response::new(stored.current_bytes.into());
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn from_anyhow(err: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "ok": false,
            "error": self.message,
        }));
        (self.status, body).into_response()
    }
}
