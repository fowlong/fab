mod pdf;
mod types;
mod util;

use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};
use axum::{
    extract::{multipart::MultipartError, Either, Multipart, Path, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{
    extract::{extract_page0, Page0Cache},
    loader::{load, LoadedDoc},
    patch,
};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<DocStore>>,
}

struct DocStore {
    docs: HashMap<String, LoadedDoc>,
}

impl Default for DocStore {
    fn default() -> Self {
        Self {
            docs: HashMap::new(),
        }
    }
}

impl DocStore {
    fn insert(&mut self, id: String, doc: LoadedDoc) {
        self.docs.insert(id, doc);
    }

    fn get(&self, id: &str) -> Option<&LoadedDoc> {
        self.docs.get(id)
    }

    fn get_mut(&mut self, id: &str) -> Option<&mut LoadedDoc> {
        self.docs.get_mut(id)
    }
}

#[derive(Debug, serde::Deserialize)]
struct OpenBase64Payload {
    data: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(AppState::default())
        .layer(cors);

    let addr: SocketAddr = ([127, 0, 0, 1], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
async fn open_document(
    State(state): State<AppState>,
    payload: Either<Multipart, Json<OpenBase64Payload>>,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = match payload {
        Either::Left(mut multipart) => read_multipart(&mut multipart).await?,
        Either::Right(Json(body)) => decode_base64_pdf(&body.data)?,
    };
    if bytes.is_empty() {
        return Err(ApiError::InvalidInput("empty PDF payload".into()));
    }

    let doc_id = new_doc_id();
    let dir = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, &bytes)?;
    let doc = load(bytes, path)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), doc);
    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let cache = ensure_page_cache(doc)?;
    Ok(Json(cache.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    patch::apply_patches(doc, &ops)?;
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
        HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn read_multipart(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(ApiError::InvalidInput("missing file field".into()))
}

fn decode_base64_pdf(data: &str) -> Result<Vec<u8>, ApiError> {
    let trimmed = data
        .split_once(',')
        .map(|(_, tail)| tail)
        .unwrap_or(data)
        .trim();
    BASE64
        .decode(trimmed)
        .map_err(|err| ApiError::InvalidInput(format!("invalid base64 payload: {err}")))
}

fn ensure_page_cache(doc: &mut LoadedDoc) -> Result<&Page0Cache, ApiError> {
    if doc.page0_cache.is_none() {
        let cache = extract_page0(&doc.doc)?;
        doc.page0_cache = Some(cache);
    }
    Ok(doc.page0_cache.as_ref().unwrap())
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("invalid request: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::InvalidInput(message) => (StatusCode::BAD_REQUEST, message).into_response(),
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

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Internal(err.into())
    }
}
