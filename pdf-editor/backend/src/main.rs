use axum::extract::Multipart;
use axum::response::IntoResponse;
use axum::{extract::Path, extract::State, routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use pdf::loader::PdfStore;

#[derive(Clone)]
struct AppState {
    store: PdfStore,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: PdfStore::default(),
    };

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .route("/api/patch/:doc_id", post(apply_patch))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    tracing::info!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn open_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, AppError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            let data = field.bytes().await?;
            let doc_id = state.store.insert(data.to_vec()).await;
            return Ok(Json(OpenResponse { doc_id }));
        }
    }
    Err(AppError::BadRequest("file field missing".into()))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<types::DocumentIr>, AppError> {
    let doc = state.store.get(&doc_id).await?;
    let ir = pdf::extract::extract_document_ir(&doc.bytes)?;
    Ok(Json(ir))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Vec<u8>, AppError> {
    let doc = state.store.get(&doc_id).await?;
    Ok(doc.bytes.clone())
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(patch_ops): Json<Vec<types::PatchOperation>>,
) -> Result<Json<types::PatchResponse>, AppError> {
    let mut doc = state.store.get(&doc_id).await?;
    let response = pdf::patch::apply_patch(&mut doc, &patch_ops)?;
    state.store.replace(doc_id, doc).await;
    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Pdf(#[from] pdf::PdfError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AppError {
    fn status_code(&self) -> axum::http::StatusCode {
        match self {
            AppError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::Pdf(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

type Result<T> = std::result::Result<T, AppError>;

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(serde_json::json!({
            "ok": false,
            "error": self.to_string(),
        }));
        (self.status_code(), body).into_response()
    }
}
