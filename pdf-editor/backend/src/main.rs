use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::loader::PdfStore;
use types::{DocumentIr, PatchOp};

#[derive(Clone)]
struct AppState {
    store: Arc<PdfStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(PdfStore::default());
    let app_state = AppState { store };

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn open_pdf(State(state): State<AppState>, mut multipart: Multipart) -> Result<Json<OpenResponse>, ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(ApiError::from)? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(ApiError::from)?;
            let doc_id = Uuid::new_v4().to_string();
            state.store.insert(doc_id.clone(), data.to_vec()).await?;
            return Ok(Json(OpenResponse { doc_id }));
        }
    }

    Err(ApiError::new(StatusCode::BAD_REQUEST, "missing file upload"))
}

async fn get_ir(State(state): State<AppState>, Path(doc_id): Path<String>) -> Result<Json<DocumentIr>, ApiError> {
    let ir = state.store.load_ir(&doc_id).await?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<pdf::patch::PatchResponse>, ApiError> {
    let response = state.store.apply_patch(&doc_id, ops).await?;
    Ok(Json(response))
}

async fn download_pdf(State(state): State<AppState>, Path(doc_id): Path<String>) -> Result<Response, ApiError> {
    let bytes = state.store.read_latest(&doc_id).await?;
    let mut response = Response::new(bytes.into());
    response
        .headers_mut()
        .insert(axum::http::header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/pdf"));
    Ok(response)
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    }
}

impl From<axum::Error> for ApiError {
    fn from(error: axum::Error) -> Self {
        Self::new(StatusCode::BAD_REQUEST, error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse { message: self.message });
        (self.status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
