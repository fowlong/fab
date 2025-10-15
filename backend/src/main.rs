use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use tokio::signal;
use tracing::{error, info};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::loader::{load_document, DocumentStore};
use pdf::patch::apply_patch_ops;
use types::{DocumentIr, PatchOperation};

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = AppState {
        store: Arc::new(DocumentStore::new()),
    };

    let app = axum::Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(app_state);

    let addr: SocketAddr = "0.0.0.0:8787".parse().unwrap();
    info!("starting server", %addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[derive(Debug, serde::Deserialize)]
struct OpenRequest {
    #[serde(rename = "pdfBase64")]
    pdf_base64: String,
}

#[derive(Debug, serde::Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: Uuid,
}

async fn open_document(
    State(app_state): State<AppState>,
    Json(payload): Json<OpenRequest>,
) -> Result<Json<OpenResponse>, AppError> {
    let pdf_bytes = base64::decode(payload.pdf_base64).map_err(AppError::bad_request)?;
    let document = load_document(pdf_bytes)
        .await
        .map_err(AppError::internal_server_error)?;

    let doc_id = Uuid::new_v4();
    app_state.store.insert(doc_id, document).await;

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(app_state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<DocumentIr>, AppError> {
    let document = app_state
        .store
        .get(&doc_id)
        .await
        .ok_or_else(|| AppError::not_found("document"))?;
    Ok(Json(document.ir))
}

async fn apply_patch(
    State(app_state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<types::PatchResponse>, AppError> {
    let document = app_state
        .store
        .get(&doc_id)
        .await
        .ok_or_else(|| AppError::not_found("document"))?;

    let (next_ir, response, updated_pdf) = apply_patch_ops(&document.ir, &ops)
        .await
        .map_err(AppError::internal_server_error)?;

    app_state
        .store
        .update_pdf(&doc_id, updated_pdf.clone(), next_ir)
        .await
        .map_err(AppError::internal_server_error)?;

    Ok(Json(response))
}

async fn download_pdf(
    State(app_state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let document = app_state
        .store
        .get(&doc_id)
        .await
        .ok_or_else(|| AppError::not_found("document"))?;

    let pdf_bytes: Vec<u8> = (*document.latest_pdf).clone();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/pdf")
        .body(axum::body::boxed(axum::body::Full::from(pdf_bytes)))
        .map_err(AppError::internal_server_error)?;
    Ok(response)
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal server error: {0}")]
    Internal(String),
}

impl AppError {
    fn bad_request<E: std::fmt::Display>(err: E) -> Self {
        AppError::BadRequest(err.to_string())
    }

    fn not_found(entity: &str) -> Self {
        AppError::NotFound(entity.to_string())
    }

    fn internal_server_error<E: std::fmt::Display>(err: E) -> Self {
        error!("internal error: {err}");
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, message).into_response()
    }
}
