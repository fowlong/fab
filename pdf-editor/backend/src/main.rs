mod pdf;
mod types;
mod util;

use std::{collections::HashMap, sync::Arc};

use axum::extract::Multipart;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use parking_lot::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::types::{DocumentIR, PatchOp, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, DocumentEntry>>>,
}

struct DocumentEntry {
    bytes: Vec<u8>,
    ir: DocumentIR,
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("Missing file in multipart form data")]
    MissingFile,
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::MissingFile => StatusCode::BAD_REQUEST,
            ApiError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let payload = serde_json::json!({
            "ok": false,
            "error": self.to_string()
        });
        (status, Json(payload)).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/open", post(open_handler))
        .route("/api/ir/:doc_id", get(ir_handler))
        .route("/api/patch/:doc_id", post(patch_handler))
        .route("/api/pdf/:doc_id", get(pdf_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8787)).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(anyhow::Error::from)? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(anyhow::Error::from)?;
            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or(ApiError::MissingFile)?;
    let document = pdf::loader::load_document(&bytes).map_err(ApiError::Other)?;
    let ir = pdf::extract::extract_ir(&document).map_err(ApiError::Other)?;

    let doc_id = Uuid::new_v4().to_string();
    state.store.write().insert(
        doc_id.clone(),
        DocumentEntry {
            bytes,
            ir: ir.clone(),
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn ir_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let guard = state.store.read();
    let entry = guard
        .get(&doc_id)
        .ok_or_else(|| ApiError::Other(anyhow::anyhow!("document not found")))?;
    Ok(Json(entry.ir.clone()))
}

async fn patch_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut guard = state.store.write();
    let entry = guard
        .get_mut(&doc_id)
        .ok_or_else(|| ApiError::Other(anyhow::anyhow!("document not found")))?;

    let updated_pdf =
        pdf::patch::apply_patch(&entry.bytes, &entry.ir, &ops).map_err(ApiError::Other)?;
    entry.bytes = updated_pdf.clone();
    // For now we do not mutate the IR; in a full implementation patch::apply_patch would.

    let base64_pdf = format!(
        "data:application/pdf;base64,{}",
        STANDARD.encode(&updated_pdf)
    );
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(base64_pdf),
        remap: HashMap::new(),
        error: None,
    }))
}

async fn pdf_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, ApiError> {
    let guard = state.store.read();
    let entry = guard
        .get(&doc_id)
        .ok_or_else(|| ApiError::Other(anyhow::anyhow!("document not found")))?;

    let mut response = Response::new(Body::from(entry.bytes.clone()));
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    Ok(response)
}

#[derive(serde::Serialize)]
struct OpenResponse {
    doc_id: String,
}
