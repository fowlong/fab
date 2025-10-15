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

use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use pdf::extract::{self, PageExtraction};
use pdf::loader;
use pdf::patch::{self, PatchResult};
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    doc: lopdf::Document,
    cache: Cache,
}

#[derive(Default)]
struct Cache {
    page0: Option<PageExtraction>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenJson {
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
        .allow_methods([Method::GET, Method::POST])
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
    multipart: Option<Multipart>,
    json: Option<Json<OpenJson>>,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut pdf_bytes = Vec::new();

    if let Some(mut form) = multipart {
        while let Some(field) = form.next_field().await? {
            if field.name() == Some("file") {
                pdf_bytes = field.bytes().await?.to_vec();
                break;
            }
        }
    }

    if pdf_bytes.is_empty() {
        if let Some(Json(payload)) = json {
            pdf_bytes = BASE64.decode(payload.data.as_bytes())?;
        }
    }

    if pdf_bytes.is_empty() {
        return Err(ApiError::BadRequest("PDF payload missing".into()));
    }

    let doc = loader::parse_document(&pdf_bytes)?;
    let doc_id = new_doc_id();
    let path = PathBuf::from(format!("/tmp/fab/{doc_id}.pdf"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&path, &pdf_bytes).await?;

    let entry = LoadedDoc {
        path,
        bytes: pdf_bytes,
        doc,
        cache: Cache::default(),
    };

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), entry);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if entry.cache.page0.is_none() {
        let extraction = extract::extract_page0(&entry.doc, None)?;
        entry.cache.page0 = Some(extraction);
    }
    let ir = entry
        .cache
        .page0
        .as_ref()
        .map(|page| page.ir.clone())
        .unwrap_or_default();
    Ok(Json(ir))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if entry.cache.page0.is_none() {
        let extraction = extract::extract_page0(&entry.doc, None)?;
        entry.cache.page0 = Some(extraction);
    }
    let extraction = entry.cache.page0.as_ref().unwrap();
    let PatchResult { bytes, extraction } =
        patch::apply_patches(&entry.doc, &entry.bytes, extraction, &ops)?;
    entry.bytes = bytes;
    entry.doc = loader::parse_document(&entry.bytes)?;
    entry.cache.page0 = Some(extraction);
    fs::write(&entry.path, &entry.bytes).await?;

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
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Pdf(#[from] anyhow::Error),
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
            ApiError::Decode(err) => {
                tracing::error!(error = %err, "base64 decode error");
                (StatusCode::BAD_REQUEST, "invalid base64 payload").into_response()
            }
            ApiError::Io(err) => {
                tracing::error!(error = %err, "filesystem error");
                (StatusCode::INTERNAL_SERVER_ERROR, "storage error").into_response()
            }
            ApiError::Pdf(err) => {
                tracing::error!(error = %err, "pdf processing error");
                (StatusCode::INTERNAL_SERVER_ERROR, "pdf processing error").into_response()
            }
        }
    }
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}
