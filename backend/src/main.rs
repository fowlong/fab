mod pdf;
mod types;
mod util;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Multipart, Path, Request, State};
use axum::http::{header, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::loader::LoadedDocument;
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
const SAMPLE_PDF: &[u8] = include_bytes!("../../e2e/sample.pdf");

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDocument>>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenJsonPayload {
    data: String,
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
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<header::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = ([127, 0, 0, 1], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = parse_pdf_payload(req).await?;
    let pdf_bytes = if bytes.is_empty() {
        SAMPLE_PDF.to_vec()
    } else {
        bytes
    };

    let doc_id = new_doc_id();
    let path = persist_initial_bytes(&doc_id, &pdf_bytes)?;

    let document = LoadedDocument::new(pdf_bytes, path)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), document);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let doc = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(doc.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    pdf::patch::apply_patches(doc, &ops)?;
    let encoded = format!("data:application/pdf;base64,{}", BASE64.encode(&doc.bytes));
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
    }))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let store = state.store.read().await;
    let doc = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(doc.bytes.clone().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn parse_pdf_payload(mut req: Request) -> Result<Vec<u8>, ApiError> {
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(req, &())
            .await
            .map_err(ApiError::Multipart)?;
        let mut buffer = Vec::new();
        while let Some(field) = multipart.next_field().await.map_err(ApiError::Multipart)? {
            if field.name() == Some("file") {
                buffer = field.bytes().await.map_err(ApiError::Multipart)?.to_vec();
                break;
            }
        }
        Ok(buffer)
    } else if !content_type.is_empty() {
        let body = hyper::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(err.into()))?;
        if body.is_empty() {
            return Ok(Vec::new());
        }
        let payload: OpenJsonPayload = serde_json::from_slice(&body)
            .map_err(|_| ApiError::BadRequest("invalid JSON payload".into()))?;
        BASE64
            .decode(payload.data.trim())
            .map_err(|_| ApiError::BadRequest("invalid base64".into()))
    } else {
        let body = hyper::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(err.into()))?;
        Ok(body.to_vec())
    }
}

fn persist_initial_bytes(doc_id: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let dir = PathBuf::from("/tmp/fab");
    std::fs::create_dir_all(&dir).map_err(|err| ApiError::Internal(err.into()))?;
    let path = dir.join(format!("{doc_id}.pdf"));
    std::fs::write(&path, bytes).map_err(|err| ApiError::Internal(err.into()))?;
    Ok(path)
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::Multipart(err) => {
                warn!(error = %err, "multipart error");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::Internal(err) => {
                warn!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
