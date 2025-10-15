use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::types::{DocumentId, DocumentIr, PatchOp, PatchResponse};

mod pdf;
mod types;
mod util;

#[derive(Clone, Default)]
struct AppState {
    docs: Arc<RwLock<HashMap<DocumentId, StoredDocument>>>,
}

#[derive(Clone)]
struct StoredDocument {
    original: Vec<u8>,
    latest: Vec<u8>,
    ir: DocumentIr,
}

#[derive(serde::Deserialize)]
struct OpenRequest {
    /// PDF payload as a base64 string (raw, not data URL).
    pdf_base64: String,
}

#[derive(serde::Serialize)]
struct OpenResponse {
    doc_id: DocumentId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let app_state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(app_state);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    info!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

fn setup_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("valid env filter");

    fmt().with_env_filter(env_filter).init();
}

async fn open_document(
    State(state): State<AppState>,
    Json(request): Json<OpenRequest>,
) -> Result<Json<OpenResponse>, ApiError> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(request.pdf_base64.trim())
        .map_err(|err| ApiError::bad_request(format!("invalid base64 payload: {err}")))?;

    // Placeholder IR extraction: this will be replaced with a real parser in later iterations.
    let ir = DocumentIr::empty();

    let doc_id = Uuid::new_v4();
    let stored = StoredDocument {
        original: decoded.clone(),
        latest: decoded,
        ir,
    };

    state.docs.write().insert(doc_id, stored);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<DocumentId>,
) -> Result<Json<DocumentIr>, ApiError> {
    let docs = state.docs.read();
    let doc = docs
        .get(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;
    Ok(Json(doc.ir.clone()))
}

#[derive(serde::Deserialize)]
struct PatchRequest {
    ops: Vec<PatchOp>,
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<DocumentId>,
    Json(body): Json<PatchRequest>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut docs = state.docs.write();
    let doc = docs
        .get_mut(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;

    // For now, just echo the operations and leave the PDF untouched.
    // Future revisions will route through pdf::patch to mutate the content streams.
    if let Err(err) = pdf::patch::validate_patch_ops(&body.ops) {
        return Err(ApiError::bad_request(err.to_string()));
    }

    let response = PatchResponse {
        ok: true,
        updated_pdf: Some(base64::engine::general_purpose::STANDARD.encode(&doc.latest)),
        remap: HashMap::new(),
    };

    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<DocumentId>,
) -> Result<Response, ApiError> {
    let docs = state.docs.read();
    let doc = docs
        .get(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .body(axum::body::Full::from(doc.latest.clone()))
        .map_err(|err| ApiError::internal(err.to_string()))?;
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=edited.pdf"),
    );
    Ok(response)
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("{message}")]
    BadRequest { message: String },
    #[error("{message}")]
    NotFound { message: String },
    #[error("internal server error: {message}")]
    Internal { message: String },
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "error": self.to_string(),
        });
        (status, Json(body)).into_response()
    }
}
