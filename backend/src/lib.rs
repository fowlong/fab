pub mod pdf;
pub mod types;
pub mod util;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{
    extract::{Multipart, Path},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::sync::RwLock;
use tracing_subscriber::prelude::*;

use crate::types::{DocumentIR, PatchOperation, PatchResponse};

pub const SAMPLE_PDF: &[u8] = include_bytes!("../../e2e/sample.pdf");
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
pub struct AppState {
    store: Arc<RwLock<HashMap<String, DocumentEntry>>>,
}

#[derive(Clone)]
struct DocumentEntry {
    ir: DocumentIR,
    pdf: Vec<u8>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResponse {
    pub doc_id: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .layer(Extension(state))
}

pub fn init_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
}

async fn open_document(
    Extension(state): Extension<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut pdf_bytes: Vec<u8> = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            pdf_bytes = field.bytes().await?.to_vec();
            break;
        }
    }
    if pdf_bytes.is_empty() {
        pdf_bytes = SAMPLE_PDF.to_vec();
    }

    let ir = DocumentIR::sample();
    let doc_id = new_doc_id();

    let mut store = state.store.write().await;
    store.insert(
        doc_id.clone(),
        DocumentEntry {
            ir: ir.clone(),
            pdf: pdf_bytes,
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    Extension(state): Extension<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    tracing::info!(doc_id, op_count = ops.len(), "received patch batch");
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let mut ir = entry.ir.clone();
    pdf::patch::apply_patches(&mut ir, &ops)?;
    entry.ir = ir;

    let encoded = format!("data:application/pdf;base64,{}", BASE64.encode(&entry.pdf));

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
        message: Some("Patch handling is stubbed in the MVP backend".into()),
    }))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<Response, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(entry.pdf.clone().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::Multipart(err) => {
                tracing::error!(error = %err, "multipart error");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_ir_contains_objects() {
        let ir = DocumentIR::sample();
        assert_eq!(ir.pages.len(), 1);
        assert_eq!(ir.pages[0].objects.len(), 2);
    }
}
