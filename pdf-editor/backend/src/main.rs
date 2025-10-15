use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use crate::pdf::{loader::PdfLoader, patch::PatchEngine};
use crate::types::{DocumentId, PatchOperation};

#[derive(Clone, Default)]
struct AppState {
    loader: Arc<PdfLoader>,
    patches: Arc<PatchEngine>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8787));
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn open_document(State(state): State<AppState>) -> impl IntoResponse {
    let doc_id = state.loader.open_placeholder().await;
    Json(OpenResponse { doc_id })
}

async fn fetch_ir(Path(doc_id): Path<DocumentId>, State(state): State<AppState>) -> impl IntoResponse {
    match state.loader.fetch_ir(&doc_id).await {
        Ok(ir) => Json(ir).into_response(),
        Err(error) => {
            tracing::error!(?error, "failed to fetch IR");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn apply_patch(
    Path(doc_id): Path<DocumentId>,
    State(state): State<AppState>,
    Json(patch): Json<Vec<PatchOperation>>,
) -> impl IntoResponse {
    match state.patches.apply(&doc_id, patch).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            tracing::error!(?error, "patch application failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn download_pdf(Path(doc_id): Path<DocumentId>, State(state): State<AppState>) -> impl IntoResponse {
    match state.loader.download(&doc_id).await {
        Ok(bytes) => ([("content-type", "application/pdf")], bytes).into_response(),
        Err(error) => {
            tracing::error!(?error, "download failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct OpenResponse {
    doc_id: DocumentId,
}
