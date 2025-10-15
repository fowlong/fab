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

use anyhow::{anyhow, Context};
use axum::{
    extract::{multipart::MultipartError, Either, Multipart, Path as AxumPath, State},
    http::{header, HeaderValue, Method, StatusCode},
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
        extract::{self, ExtractionResult, PageCache},
        patch, write,
    },
    types::{DocumentIR, PatchOperation, PatchResponse},
};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    doc: lopdf::Document,
    cache: Option<DocumentCache>,
}

#[derive(Clone)]
struct DocumentCache {
    ir: DocumentIR,
    page_cache: PageCache,
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

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

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
        .allow_origin(
            HeaderValue::from_str("http://localhost:5173").expect("valid localhost origin header"),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

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
    payload: Either<Json<OpenJsonPayload>, Multipart>,
) -> Result<Json<OpenResponse>, ApiError> {
    let pdf_bytes = match payload {
        Either::Left(Json(OpenJsonPayload { data })) => decode_base64_payload(&data)?,
        Either::Right(mut multipart) => {
            let mut bytes = Vec::new();
            while let Some(field) = multipart.next_field().await? {
                if field.name() == Some("file") {
                    bytes = field.bytes().await?.to_vec();
                    break;
                }
            }
            if bytes.is_empty() {
                return Err(ApiError::InvalidRequest(
                    "multipart payload missing 'file' field".into(),
                ));
            }
            bytes
        }
    };

    if pdf_bytes.is_empty() {
        return Err(ApiError::InvalidRequest("empty PDF payload".into()));
    }

    let doc_id = new_doc_id();
    let path = write_to_tmp(&doc_id, &pdf_bytes).await?;

    let parse_bytes = pdf_bytes.clone();
    let (doc, extraction) = tokio::task::spawn_blocking(
        move || -> anyhow::Result<(lopdf::Document, ExtractionResult)> {
            let mut doc = pdf::loader::parse_document(&parse_bytes)?;
            let extraction = extract::extract_page_zero(&doc)?;
            Ok((doc, extraction))
        },
    )
    .await
    .map_err(|err| ApiError::Internal(err.into()))??;

    let cache = DocumentCache {
        ir: extraction.ir,
        page_cache: extraction.page_cache,
    };

    let mut store = state.store.write().await;
    store.insert(
        doc_id.clone(),
        LoadedDoc {
            path,
            bytes: pdf_bytes,
            doc,
            cache: Some(cache),
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    ensure_cache(entry)?;
    let ir = entry
        .cache
        .as_ref()
        .map(|cache| cache.ir.clone())
        .ok_or(ApiError::Internal(anyhow!("cache initialisation failed")))?;
    Ok(Json(ir))
}

async fn apply_patch(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    ensure_cache(entry)?;

    let cache = entry
        .cache
        .as_mut()
        .ok_or(ApiError::Internal(anyhow!("cache not initialised")))?;

    let page_patch = patch::apply_patches(&mut cache.page_cache, &ops)?;
    let updated_bytes = if let Some(page_patch) = page_patch {
        let bytes = write::incremental_update(&mut entry.doc, &entry.bytes, page_patch)?;
        entry.bytes = bytes.clone();
        bytes
    } else {
        entry.bytes.clone()
    };

    let extraction = extract::extract_page_zero(&entry.doc)?;
    entry.cache = Some(DocumentCache {
        ir: extraction.ir,
        page_cache: extraction.page_cache,
    });

    fs::write(&entry.path, &updated_bytes)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;

    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&updated_bytes)
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
    let mut response = Response::new(entry.bytes.clone().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

fn ensure_cache(entry: &mut LoadedDoc) -> Result<(), ApiError> {
    if entry.cache.is_none() {
        let extraction = extract::extract_page_zero(&entry.doc)?;
        entry.cache = Some(DocumentCache {
            ir: extraction.ir,
            page_cache: extraction.page_cache,
        });
    }
    Ok(())
}

async fn write_to_tmp(doc_id: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let dir = Path::new("/tmp/fab");
    fs::create_dir_all(dir)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    let path = dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, bytes)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    Ok(path)
}

fn decode_base64_payload(data: &str) -> Result<Vec<u8>, ApiError> {
    let payload = if let Some(index) = data.find(',') {
        &data[index + 1..]
    } else {
        data
    };
    BASE64
        .decode(payload)
        .map_err(|err| ApiError::InvalidRequest(format!("base64 decode failed: {err}")))
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
    InvalidRequest(String),
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
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
