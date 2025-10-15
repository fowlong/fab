use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use types::{DocumentIr, PatchOp, PatchResponse};

#[derive(Default)]
struct AppState {
    documents: RwLock<HashMap<String, Arc<DocumentState>>>,
}

struct DocumentState {
    original: Vec<u8>,
    latest: RwLock<Vec<u8>>,
    ir: RwLock<DocumentIr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(Arc::clone(&state))
        .layer(CorsLayer::new().allow_origin(Any));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8787));
    info!(%addr, "Starting server");
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct OpenResponse {
    doc_id: String,
}

async fn open_pdf(
    State(app_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, AppError> {
    let mut file_bytes = None;
    while let Some(field) = multipart.next_field().await.map_err(AppError::bad_request)? {
        if let Some(name) = field.name() {
            if name == "file" {
                file_bytes = Some(field.bytes().await.map_err(AppError::bad_request)?.to_vec());
                break;
            }
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| AppError::bad_request("missing file"))?;

    let doc_id = uuid::Uuid::new_v4().to_string();
    let placeholder_ir = DocumentIr::empty();
    let doc_state = Arc::new(DocumentState {
        original: file_bytes.clone(),
        latest: RwLock::new(file_bytes.clone()),
        ir: RwLock::new(placeholder_ir),
    });

    app_state
        .documents
        .write()
        .await
        .insert(doc_id.clone(), doc_state);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(app_state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIr>, AppError> {
    let documents = app_state.documents.read().await;
    let doc = documents
        .get(&doc_id)
        .ok_or_else(|| AppError::not_found("document not found"))?;
    let ir = doc.ir.read().await.clone();
    Ok(Json(ir))
}

async fn apply_patch(
    State(app_state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
    Json(patch_ops): Json<Vec<PatchOp>>,
) -> Result<Json<PatchResponse>, AppError> {
    let documents = app_state.documents.read().await;
    let doc = documents
        .get(&doc_id)
        .ok_or_else(|| AppError::not_found("document not found"))?;

    if patch_ops.is_empty() {
        return Ok(Json(PatchResponse::noop()));
    }

    // Placeholder behaviour: simply acknowledge without mutating the PDF.
    let mut latest = doc.latest.write().await;
    let current = latest.clone();
    drop(latest);

    let mut ir = doc.ir.write().await;
    pdf::patch::apply_noop(&mut ir.pages, &patch_ops);

    Ok(Json(PatchResponse::ack(current)))
}

async fn download_pdf(
    State(app_state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
) -> Result<Response, AppError> {
    let documents = app_state.documents.read().await;
    let doc = documents
        .get(&doc_id)
        .ok_or_else(|| AppError::not_found("document not found"))?;
    let latest = doc.latest.read().await.clone();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/pdf")
        .body(axum::body::Full::from(latest))
        .unwrap())
}

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

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(status = ?self.status, message = %self.message, "request failed");
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}
