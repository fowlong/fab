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

use anyhow::Context;
use axum::{
    extract::{Either, Multipart, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lopdf::Document;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{extract::ExtractedDocument, loader, patch};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<DocStore>>,
}

#[derive(Default)]
struct DocStore {
    docs: HashMap<String, LoadedDoc>,
}

struct LoadedDoc {
    bytes: Vec<u8>,
    path: PathBuf,
    document: Document,
    cache: Option<ExtractedDocument>,
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

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let state = AppState {
        store: Arc::new(RwLock::new(DocStore::default())),
    };

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
    payload: Either<Json<OpenJsonPayload>, Multipart>,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut pdf_bytes = match payload {
        Either::A(Json(json)) => {
            decode_payload(&json.data).map_err(|err| ApiError::BadRequest(err.to_string()))?
        }
        Either::B(mut multipart) => {
            let mut data = Vec::new();
            while let Some(field) = multipart.next_field().await? {
                if field.name() == Some("file") {
                    data = field.bytes().await?.to_vec();
                    break;
                }
            }
            if data.is_empty() {
                return Err(ApiError::BadRequest("file field missing".into()));
            }
            data
        }
    };

    if pdf_bytes.is_empty() {
        return Err(ApiError::BadRequest("empty PDF payload".into()));
    }

    let doc_id = new_doc_id();
    let tmp_dir = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&tmp_dir).await?;
    let path = tmp_dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, &pdf_bytes).await?;

    let document = loader::parse_document(&pdf_bytes)?;
    let extraction = pdf::extract::extract_document(&document)?;

    let mut store = state.store.write().await;
    store.docs.insert(
        doc_id.clone(),
        LoadedDoc {
            bytes: pdf_bytes,
            path,
            document,
            cache: Some(extraction),
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let extraction = ensure_cache(entry)?;
    Ok(Json(extraction.ir.clone()))
}

async fn apply_patch(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if entry.cache.is_none() {
        entry.cache = Some(pdf::extract::extract_document(&entry.document)?);
    }
    let mut extraction = entry.cache.clone().expect("cache initialised");
    let mut bytes = entry.bytes.clone();
    let mut document = entry.document.clone();

    for op in &ops {
        let result = patch::apply_patch(&document, &bytes, &extraction, op)?;
        bytes = result.bytes;
        document = result.document;
        extraction = result.extraction;
    }

    let path = entry.path.clone();
    entry.bytes = bytes.clone();
    entry.document = document.clone();
    entry.cache = Some(extraction.clone());

    drop(store);
    fs::write(path, &bytes).await?;

    let encoded = format!("data:application/pdf;base64,{}", BASE64.encode(&bytes));

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
    let store = state.store.read().await;
    let entry = store.docs.get(&doc_id).ok_or(ApiError::NotFound)?;
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

fn decode_payload(input: &str) -> anyhow::Result<Vec<u8>> {
    let data = if let Some(idx) = input.find(',') {
        &input[idx + 1..]
    } else {
        input
    };
    let trimmed = data.trim();
    BASE64
        .decode(trimmed)
        .map_err(|err| anyhow::anyhow!("invalid base64 payload: {err}"))
}

fn ensure_cache(entry: &mut LoadedDoc) -> anyhow::Result<&ExtractedDocument> {
    if entry.cache.is_none() {
        let extraction = pdf::extract::extract_document(&entry.document)?;
        entry.cache = Some(extraction);
    }
    Ok(entry.cache.as_ref().unwrap())
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
