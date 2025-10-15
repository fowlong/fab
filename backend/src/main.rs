mod pdf;
mod types;
mod util;

use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::Context;
use axum::{
    async_trait,
    body::Body,
    extract::{FromRequest, Multipart, Path as AxumPath, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lopdf::Document;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: Document,
    cache: pdf::extract::DocumentCache,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
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
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = ([127, 0, 0, 1], 8787).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    payload: PdfPayload,
) -> Result<Json<OpenResponse>, ApiError> {
    let pdf_bytes = payload.0;
    let doc_id = new_doc_id();
    let path = persist_pdf(&doc_id, &pdf_bytes).await?;

    let document = pdf::loader::parse_document(&pdf_bytes)?;
    let cache = pdf::extract::extract_document(&document)?;

    let mut store = state.store.write().await;
    store.insert(
        doc_id.clone(),
        LoadedDoc {
            path,
            bytes: pdf_bytes,
            document,
            cache,
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.cache.ir.clone()))
}

async fn apply_patch(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;

    let outcome = pdf::patch::apply_patches(&entry.bytes, &entry.document, &entry.cache, &ops)?;
    persist_existing(&entry.path, &outcome.bytes).await?;

    entry.bytes = outcome.bytes.clone();
    entry.document = outcome.document;
    entry.cache = outcome.cache;

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
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(Body::from(entry.bytes.clone()));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

async fn persist_pdf(doc_id: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let base = Path::new("/tmp/fab");
    tokio::fs::create_dir_all(base)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    let path = base.join(format!("{doc_id}.pdf"));
    persist_existing(&path, bytes).await?;
    Ok(path)
}

async fn persist_existing(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    tokio::fs::write(path, bytes)
        .await
        .map_err(|err| ApiError::Internal(err.into()))
}

struct PdfPayload(Vec<u8>);

#[async_trait]
impl<S> FromRequest<S> for PdfPayload
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(mut request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.starts_with("multipart/form-data") {
            let mut multipart = Multipart::from_request(request, state)
                .await
                .map_err(ApiError::Multipart)?;
            while let Some(field) = multipart.next_field().await.map_err(ApiError::Multipart)? {
                if field.name() == Some("file") {
                    let bytes = field.bytes().await.map_err(ApiError::Multipart)?;
                    return Ok(PdfPayload(bytes.to_vec()));
                }
            }
            Err(ApiError::BadRequest("file field missing".into()))
        } else {
            let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
                .await
                .map_err(|err| ApiError::Internal(err.into()))?;
            let upload: JsonUpload = serde_json::from_slice(&body_bytes)
                .map_err(|_| ApiError::BadRequest("invalid JSON payload".into()))?;
            let data = match upload {
                JsonUpload::Data { data } => data,
                JsonUpload::Inline(data) => data,
            };
            let cleaned = data
                .split_once(",")
                .map(|(_, tail)| tail.to_string())
                .unwrap_or(data);
            let decoded = BASE64
                .decode(cleaned.as_bytes())
                .map_err(|_| ApiError::BadRequest("invalid base64 data".into()))?;
            Ok(PdfPayload(decoded))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum JsonUpload {
    Data { data: String },
    Inline(String),
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
                tracing::warn!(error = %err, "multipart parsing failed");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
