use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, response::IntoResponse, routing::{get, post}, Json, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use crate::pdf::loader::PdfStore;
use crate::types::{DocumentIr, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<PdfStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let store = Arc::new(PdfStore::default());
    let state = AppState { store };

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8787".to_string())
        .parse()?;
    tracing::info!(?addr, "Starting server");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn open_pdf(State(state): State<AppState>, payload: axum::extract::Multipart) -> Result<Json<OpenResponse>, ApiError> {
    let doc_id = state.store.open(payload).await?;
    Ok(Json(OpenResponse { doc_id }))
}

async fn fetch_ir(State(state): State<AppState>, axum::extract::Path(doc_id): axum::extract::Path<String>) -> Result<Json<DocumentIr>, ApiError> {
    let ir = state.store.fetch_ir(&doc_id).await?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let response = state.store.apply_patch(&doc_id, ops).await?;
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let data = state.store.download(&doc_id).await?;
    Ok(axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/pdf")
        .body(axum::body::Body::from(data))
        .unwrap())
}

#[derive(serde::Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: String,
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl<E> From<E> for ApiError
where
    anyhow::Error: From<E>,
{
    fn from(value: E) -> Self {
        ApiError::Anyhow(value.into())
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::Anyhow(err) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };
        let payload = serde_json::json!({ "error": message });
        (status, Json(payload)).into_response()
    }
}
