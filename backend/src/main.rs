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
    extract::{FromRequest, Multipart, Request, State},
    http::{self, header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    pdf::{
        extract::extract_page_zero,
        loader,
        patch::{self, PatchState},
    },
    types::{DocumentIR, PatchOperation, PatchResponse},
};

const SAMPLE_PDF: &[u8] = include_bytes!("../../e2e/sample.pdf");
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<DocStore>,
}

struct DocStore {
    inner: RwLock<HashMap<String, LoadedDoc>>,
}

impl DocStore {
    fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: lopdf::Document,
    ir: DocumentIR,
    page_cache: pdf::extract::PageCache,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenJsonPayload {
    #[serde(default)]
    base64: Option<String>,
    #[serde(default)]
    data: Option<String>,
}

struct OpenPayload(Vec<u8>);

#[async_trait]
impl<S> FromRequest<S> for OpenPayload
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(mut req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
            .unwrap_or_default();

        if content_type.starts_with("multipart/form-data") {
            let mut multipart = Multipart::from_request(req, state)
                .await
                .map_err(ApiError::from)?;
            let mut pdf_bytes = Vec::new();
            while let Some(field) = multipart.next_field().await.map_err(ApiError::from)? {
                if field.name() == Some("file") {
                    pdf_bytes = field.bytes().await.map_err(ApiError::from)?.to_vec();
                    break;
                }
            }
            if pdf_bytes.is_empty() {
                pdf_bytes = SAMPLE_PDF.to_vec();
            }
            Ok(OpenPayload(pdf_bytes))
        } else {
            let payload = Json::<OpenJsonPayload>::from_request(req, state)
                .await
                .map_err(ApiError::from)?
                .0;
            let encoded = payload
                .base64
                .or(payload.data)
                .ok_or_else(|| ApiError::BadRequest("missing base64 data".into()))?;
            let bytes = BASE64
                .decode(encoded)
                .map_err(|_| ApiError::BadRequest("invalid base64 data".into()))?;
            Ok(OpenPayload(bytes))
        }
    }
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
        store: Arc::new(DocStore::new()),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<header::HeaderValue>()?)
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(false);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    OpenPayload(mut bytes): OpenPayload,
) -> Result<Json<OpenResponse>, ApiError> {
    if bytes.is_empty() {
        bytes = SAMPLE_PDF.to_vec();
    }

    let document = loader::parse_document(&bytes).map_err(ApiError::Internal)?;
    let extraction = extract_page_zero(&document).map_err(ApiError::Internal)?;
    let doc_id = new_doc_id();

    let path = ensure_tmp_path(&doc_id).await.map_err(ApiError::Internal)?;
    fs::write(&path, &bytes).await.map_err(ApiError::Internal)?;

    let loaded = LoadedDoc {
        path,
        bytes: bytes.clone(),
        document,
        ir: extraction.ir.clone(),
        page_cache: extraction.page_cache,
    };

    let mut store = state.store.inner.write().await;
    store.insert(doc_id.clone(), loaded);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.inner.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.ir.clone()))
}

async fn apply_patch(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.inner.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;

    let patch_state = PatchState::new(
        entry.document.clone(),
        entry.bytes.clone(),
        entry.ir.clone(),
        entry.page_cache.clone(),
    );
    let updated = patch::apply_patches(patch_state, &ops).map_err(ApiError::Internal)?;

    entry.document = updated.document;
    entry.bytes = updated.bytes.clone();
    entry.ir = updated.ir.clone();
    entry.page_cache = updated.page_cache;

    fs::write(&entry.path, &entry.bytes)
        .await
        .map_err(ApiError::Internal)?;

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
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Response, ApiError> {
    let store = state.store.inner.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(entry.bytes.clone().into());
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

async fn ensure_tmp_path(doc_id: &str) -> anyhow::Result<PathBuf> {
    let base = Path::new("/tmp/fab");
    fs::create_dir_all(base).await?;
    Ok(base.join(format!("{doc_id}.pdf")))
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
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        ApiError::Multipart(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::Multipart(err) => {
                tracing::error!(error = %err, "multipart error");
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::JsonRejection(err) => {
                tracing::error!(error = %err, "json error");
                (StatusCode::BAD_REQUEST, "invalid json payload").into_response()
            }
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
