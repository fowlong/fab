mod pdf;
mod types;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, atomic::Ordering, Arc};

use anyhow::Context;
use axum::body::Bytes;
use axum::extract::Either;
use axum::extract::Json;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::pdf::extract::{build_document_ir, extract_page0};
use crate::pdf::loader::{load_document, LoadedDoc};
use crate::pdf::patch::{apply_patches, PatchResult};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Base64OpenRequest {
    #[serde(alias = "pdfBase64", alias = "data")]
    base64_pdf: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    payload: Either<Json<Base64OpenRequest>, Multipart>,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = match payload {
        Either::Left(Json(data)) => decode_base64(&data.base64_pdf)?,
        Either::Right(mut multipart) => extract_file_from_multipart(&mut multipart).await?,
    };

    if bytes.is_empty() {
        return Err(ApiError::BadRequest("no PDF supplied".into()));
    }

    let doc_id = new_doc_id();
    let path = ensure_doc_path(&doc_id).await?;
    fs::write(&path, &bytes)
        .await
        .with_context(|| format!("failed to persist {doc_id}"))?;
    let loaded = load_document(bytes, path)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), loaded);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if entry.cache.page0.is_none() {
        let cache = extract_page0(&entry.document, &entry.page0, &HashMap::new())?;
        entry.cache.page0 = Some(cache);
    }
    let cache = entry.cache.page0.as_ref().unwrap();
    Ok(Json(build_document_ir(cache)))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let PatchResult { bytes, .. } = apply_patches(entry, &ops)?;
    fs::write(&entry.path, &bytes)
        .await
        .with_context(|| format!("failed to update {doc_id}"))?;
    let encoded = format!("data:application/pdf;base64,{}", encode_base64(&bytes));
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(Bytes::from(entry.bytes.clone()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn extract_file_from_multipart(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(ApiError::BadRequest("file field missing".into()))
}

async fn ensure_doc_path(doc_id: &str) -> Result<PathBuf, ApiError> {
    let dir = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&dir).await?;
    Ok(dir.join(format!("{doc_id}.pdf")))
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("internal error: {err}"),
            )
                .into_response(),
        }
    }
}

fn decode_base64(value: &str) -> Result<Vec<u8>, ApiError> {
    let trimmed = value.trim();
    let content = trimmed
        .strip_prefix("data:application/pdf;base64,")
        .unwrap_or(trimmed);
    let mut clean = Vec::new();
    for byte in content.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        clean.push(byte);
    }
    if clean.len() % 4 != 0 {
        return Err(ApiError::BadRequest("invalid base64 length".into()));
    }
    let mut output = Vec::new();
    for chunk in clean.chunks(4) {
        let mut vals = [0u8; 4];
        let mut padding = 0;
        for (idx, &b) in chunk.iter().enumerate() {
            vals[idx] = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => {
                    padding += 1;
                    0
                }
                _ => return Err(ApiError::BadRequest("invalid base64 character".into())),
            };
        }
        output.push((vals[0] << 2) | (vals[1] >> 4));
        if padding < 2 {
            output.push((vals[1] << 4) | (vals[2] >> 2));
        }
        if padding == 0 {
            output.push((vals[2] << 6) | vals[3]);
        }
    }
    Ok(output)
}

fn encode_base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let chunk = &data[i..i + 3];
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
        output.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        output.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        output.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        output.push(TABLE[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        output.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        output.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        output.push('=');
        output.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        output.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        output.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        output.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        output.push('=');
    }
    output
}
