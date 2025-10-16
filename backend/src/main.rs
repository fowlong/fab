mod pdf;
mod types;
mod util;

use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lopdf::Document;
use serde::Deserialize;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    pdf::{extract::DocumentCache, loader, patch},
    types::{DocumentIR, PatchOperation, PatchResponse},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: Document,
    cache: DocumentCache,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, Deserialize)]
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

    let state = AppState::default();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Json<OpenResponse>, ApiError> {
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let mut pdf_bytes = if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(req, &state)
            .await
            .map_err(ApiError::Multipart)?;
        let mut bytes = Vec::new();
        while let Some(field) = multipart.next_field().await.map_err(ApiError::Multipart)? {
            if field.name() == Some("file") {
                bytes = field.bytes().await.map_err(ApiError::Multipart)?.to_vec();
                break;
            }
        }
        bytes
    } else if content_type.starts_with("application/json") {
        let bytes = hyper::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(anyhow::anyhow!(err)))?;
        let payload: OpenJsonPayload = serde_json::from_slice(&bytes)?;
        let value = payload.data.trim();
        let base64_part = value.split(',').last().unwrap_or(value);
        BASE64.decode(base64_part).map_err(ApiError::Base64)?
    } else {
        hyper::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(anyhow::anyhow!(err)))?
            .to_vec()
    };

    if pdf_bytes.is_empty() {
        return Err(ApiError::InvalidRequest("missing PDF payload".into()));
    }

    let document = loader::parse_document(&pdf_bytes)?;
    let cache = pdf::extract::extract_document(&document)?;

    let doc_id = new_doc_id();
    let dir = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&dir).await?;
    let path = dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, &pdf_bytes).await?;

    let loaded = LoadedDoc {
        path,
        bytes: pdf_bytes,
        document,
        cache,
    };

    state.store.write().await.insert(doc_id.clone(), loaded);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.cache.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;

    let maybe_bytes = patch::apply_patches(&entry.document, &entry.cache, &entry.bytes, &ops)?;

    if let Some(new_bytes) = maybe_bytes {
        entry.bytes = new_bytes;
        entry.document = loader::parse_document(&entry.bytes)?;
        entry.cache = pdf::extract::extract_document(&entry.document)?;
        fs::write(&entry.path, &entry.bytes).await?;
    }

    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&entry.bytes)
    );

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
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(entry.bytes.clone().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::Multipart(err) => {
                tracing::warn!(error = %err, "multipart error");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::Json(err) => {
                tracing::warn!(error = %err, "json error");
                (StatusCode::BAD_REQUEST, "invalid json payload").into_response()
            }
            ApiError::Base64(err) => {
                tracing::warn!(error = %err, "base64 error");
                (StatusCode::BAD_REQUEST, "invalid base64 data").into_response()
            }
            ApiError::Io(err) => {
                tracing::error!(error = %err, "filesystem error");
                (StatusCode::INTERNAL_SERVER_ERROR, "storage error").into_response()
            }
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}
