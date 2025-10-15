mod pdf;
mod types;

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
use axum::extract::Either;
use axum::{
    extract::{multipart::MultipartError, Multipart, Path as AxumPath, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;

use crate::{
    pdf::{
        extract::{self, ExtractResult},
        loader, patch, write,
    },
    types::{DocumentIR, PatchOperation, PatchResponse},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: lopdf::Document,
    ir: DocumentIR,
    cache: extract::ExtractCache,
}

#[derive(Debug, serde::Deserialize)]
struct OpenJsonPayload {
    base64: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("invalid request payload")]
    BadRequest,
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::BadRequest => {
                (StatusCode::BAD_REQUEST, "invalid request payload").into_response()
            }
            ApiError::Multipart(_err) => {
                (StatusCode::BAD_REQUEST, "invalid multipart payload").into_response()
            }
            ApiError::Internal(_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    payload: Either<Multipart, Json<OpenJsonPayload>>,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = match payload {
        Either::Left(mut multipart) => read_multipart_bytes(&mut multipart).await?,
        Either::Right(Json(body)) => decode_base64(&body.base64)?,
    };
    if bytes.is_empty() {
        return Err(ApiError::BadRequest);
    }

    let document = loader::parse_document(&bytes)?;
    let ExtractResult { ir, cache } = extract::extract_ir(&document)?;

    let doc_id = new_doc_id();
    let path = persist_bytes(&doc_id, &bytes).await?;

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
    if ops.is_empty() {
        return Ok(Json(PatchResponse {
            ok: true,
            updated_pdf: None,
            remap: None,
        }));
    }

    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let page = entry.ir.pages.get(0).ok_or(ApiError::BadRequest)?;
    let page_cache = entry
        .cache
        .page0
        .as_mut()
        .ok_or(ApiError::Internal(anyhow!("page cache missing")))?;

    patch::apply_patches(page, page_cache, &ops)?;

    let updated_bytes = write::incremental_update(&entry.bytes, &entry.document, page_cache)?;
    persist_existing(&entry.path, &updated_bytes).await?;

    entry.bytes = updated_bytes;
    entry.document = loader::parse_document(&entry.bytes)?;
    let ExtractResult { ir, cache } = extract::extract_ir(&entry.document)?;
    entry.ir = ir;
    entry.cache = cache;

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
    let mut response = Response::new(entry.bytes.clone().into());
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

async fn read_multipart_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            let data = field.bytes().await?.to_vec();
            return Ok(data);
        }
    }
    Err(ApiError::BadRequest)
}

fn decode_base64(data: &str) -> Result<Vec<u8>, ApiError> {
    let trimmed = if let Some(index) = data.find(',') {
        &data[index + 1..]
    } else {
        data
    };
    BASE64
        .decode(trimmed.as_bytes())
        .map_err(|_| ApiError::BadRequest)
}

async fn persist_bytes(doc_id: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let dir = Path::new("/tmp/fab");
    fs::create_dir_all(dir)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    let path = dir.join(format!("{doc_id}.pdf"));
    persist_existing(&path, bytes).await?;
    Ok(path)
}

async fn persist_existing(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    fs::write(path, bytes)
        .await
        .map_err(|err| ApiError::Internal(err.into()))
}
