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

use anyhow::{anyhow, Context};
use axum::{
    body::Body,
    extract::{FromRequest, Multipart, Path, State},
    http::{header, HeaderValue, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    pdf::{extract, loader, patch},
    types::{DocumentIR, PatchOperation, PatchResponse},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, loader::LoadedDoc>>>,
    tmp_dir: Arc<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            tmp_dir: Arc::new(PathBuf::from("/tmp/fab")),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(serde::Deserialize)]
struct Base64OpenPayload {
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
        .allow_origin(
            HeaderValue::from_str("http://localhost:5173").expect("valid localhost origin"),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

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
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let pdf_bytes = if content_type.starts_with("multipart/form-data") {
        read_multipart(req, state.clone()).await?
    } else {
        read_base64(req).await?
    };

    let doc_id = new_doc_id();
    let path = loader::persist_bytes(&state.tmp_dir, &doc_id, &pdf_bytes).await?;
    let doc = loader::parse_document(&pdf_bytes)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), loader::LoadedDoc::new(pdf_bytes, doc, path));

    Ok(Json(OpenResponse { doc_id }))
}

async fn read_multipart(req: Request<Body>, state: AppState) -> Result<Vec<u8>, ApiError> {
    let mut multipart = Multipart::from_request(req, &state)
        .await
        .map_err(ApiError::Multipart)?;
    while let Some(field) = multipart.next_field().await.map_err(ApiError::Multipart)? {
        if field.name() == Some("file") {
            return field
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(ApiError::Multipart);
        }
    }
    Err(ApiError::BadRequest("missing file field".into()))
}

async fn read_base64(req: Request<Body>) -> Result<Vec<u8>, ApiError> {
    let body = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    if body.is_empty() {
        return Err(ApiError::BadRequest("empty payload".into()));
    }
    let payload: Base64OpenPayload = serde_json::from_slice(&body)
        .map_err(|err| ApiError::BadRequest(format!("invalid JSON: {err}")))?;
    decode_base64_data(&payload.data)
}

fn decode_base64_data(data: &str) -> Result<Vec<u8>, ApiError> {
    let raw = data.split_once(',').map(|(_, tail)| tail).unwrap_or(data);
    BASE64
        .decode(raw)
        .map_err(|_| ApiError::BadRequest("invalid base64 data".into()))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if entry.cache.page0.is_none() {
        let extraction = extract::extract_page0(&entry.doc)?;
        entry.cache.page0 = Some(extraction);
    }
    let page = entry
        .cache
        .page0
        .as_ref()
        .map(|cache| cache.ir.clone())
        .ok_or(ApiError::Internal(anyhow::anyhow!("page cache missing")))?;
    Ok(Json(DocumentIR { pages: vec![page] }))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let extraction = if let Some(cache) = entry.cache.page0.clone() {
        cache
    } else {
        let extracted = extract::extract_page0(&entry.doc)?;
        entry.cache.page0 = Some(extracted.clone());
        extracted
    };

    patch::apply_patches(entry, &extraction, &ops)?;

    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&entry.bytes)
    );

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
        message: None,
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
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
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Multipart(err) => {
                tracing::error!(error = %err, "multipart error");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
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
