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
    extract::{Either, Multipart, Path as AxumPath, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lopdf::Document;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{
    extract::{extract_page_zero, ExtractionCaches, PageCache},
    loader, patch, write,
};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
const TMP_DIR: &str = "/tmp/fab";

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

#[derive(Clone)]
struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: Document,
    ir: DocumentIR,
    cache: ExtractionCaches,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct Base64Upload {
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
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
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
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    payload: Either<Multipart, Json<Base64Upload>>,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = match payload {
        Either::Left(mut multipart) => read_multipart_pdf(&mut multipart).await?,
        Either::Right(Json(upload)) => decode_data_url(&upload.data)?,
    };

    if bytes.is_empty() {
        return Err(ApiError::BadRequest("PDF payload missing".into()));
    }

    fs::create_dir_all(TMP_DIR)
        .await
        .context("failed to create temp directory")?;

    let doc_id = new_doc_id();
    let path = Path::new(TMP_DIR).join(format!("{doc_id}.pdf"));
    fs::write(&path, &bytes)
        .await
        .context("failed to write PDF to temp storage")?;

    let document = loader::parse_document(&bytes).context("failed to parse PDF")?;
    let (ir, cache) = extract_page_zero(&document).context("failed to extract IR")?;

    let mut store = state.store.write().await;
    store.insert(
        doc_id.clone(),
        LoadedDoc {
            path,
            bytes,
            document,
            ir,
            cache,
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn read_multipart_pdf(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(ApiError::BadRequest(
        "multipart payload missing 'file' field".into(),
    ))
}

fn decode_data_url(data: &str) -> Result<Vec<u8>, ApiError> {
    let payload = if let Some((_, encoded)) = data.split_once(",") {
        encoded
    } else {
        data
    };
    BASE64
        .decode(payload)
        .map_err(|err| ApiError::BadRequest(format!("invalid base64 payload: {err}")))
}

async fn get_ir(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.ir.clone()))
}

async fn apply_patch(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;

    if ops.is_empty() {
        let data_url = format!(
            "data:application/pdf;base64,{}",
            BASE64.encode(&entry.bytes)
        );
        return Ok(Json(PatchResponse {
            ok: true,
            updated_pdf: Some(data_url),
            remap: None,
        }));
    }

    let page_cache: PageCache = match entry.cache.page0.clone() {
        Some(cache) => cache,
        None => {
            let (ir, cache) = extract_page_zero(&entry.document).context("failed to extract IR")?;
            entry.ir = ir;
            entry.cache = cache;
            entry
                .cache
                .page0
                .clone()
                .ok_or_else(|| ApiError::Internal(anyhow!("page cache missing")))?
        }
    };

    let new_stream = patch::apply_patches(&page_cache, &ops).context("failed to apply patches")?;
    let updated_bytes =
        write::incremental_update(&entry.document, &page_cache, &entry.bytes, new_stream)
            .context("failed to write incremental update")?;

    fs::write(&entry.path, &updated_bytes)
        .await
        .context("failed to persist updated PDF")?;

    entry.bytes = updated_bytes;
    entry.document = loader::parse_document(&entry.bytes).context("failed to parse updated PDF")?;
    let (ir, cache) = extract_page_zero(&entry.document).context("failed to refresh IR")?;
    entry.ir = ir;
    entry.cache = cache;

    let data_url = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&entry.bytes)
    );

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(data_url),
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
