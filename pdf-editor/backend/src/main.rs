use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    pdf::{extract, loader::PdfDocumentStore},
    types::{PdfOpenResponse, PdfState},
};

mod pdf;
mod types;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let store = Arc::new(PdfDocumentStore::default());

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .route("/api/patch/:doc_id", post(apply_patch))
        .with_state(store);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    info!(%addr, "listening");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

type SharedStore = Arc<PdfDocumentStore>;

async fn open_pdf(
    State(store): State<SharedStore>,
    mut multipart: Multipart,
) -> Result<Json<PdfOpenResponse>, ApiError> {
    let mut bytes = None;
    while let Some(field) = multipart.next_field().await.map_err(ApiError::from)? {
        if field.name() == Some("file") {
            let mut data = Vec::new();
            let mut reader = field;
            while let Some(chunk) = reader.chunk().await.map_err(ApiError::from)? {
                data.extend_from_slice(&chunk);
            }
            bytes = Some(data);
            break;
        }
    }

    let data = bytes.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing file"))?;
    let doc_id = Uuid::new_v4().to_string();
    let ir = extract::extract_ir(&data).map_err(ApiError::from)?;
    store.store(doc_id.clone(), data, ir).await;
    Ok(Json(PdfOpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(store): State<SharedStore>,
) -> Result<Json<PdfState>, ApiError> {
    let doc = store
        .get(&doc_id)
        .await
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "doc not found"))?;
    Ok(Json((*doc.ir).clone()))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    State(store): State<SharedStore>,
) -> Result<impl IntoResponse, ApiError> {
    let doc = store
        .get(&doc_id)
        .await
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "doc not found"))?;
    let body = axum::body::Full::from(doc.pdf_bytes.clone());
    let headers = [
        (
            axum::http::header::CONTENT_TYPE,
            "application/pdf".to_string(),
        ),
        (
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"document.pdf\"".to_string(),
        ),
    ];
    Ok((headers, body))
}

async fn apply_patch(
    Path(_doc_id): Path<String>,
    State(_store): State<SharedStore>,
    Json(_ops): Json<Vec<crate::types::PatchOperation>>,
) -> Result<Json<crate::types::PatchResponse>, ApiError> {
    let response = crate::types::PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    };
    Ok(Json(response))
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

impl From<axum::Error> for ApiError {
    fn from(err: axum::Error) -> Self {
        error!(?err, "axum error");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        error!(?err, "internal error");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
    }
}
