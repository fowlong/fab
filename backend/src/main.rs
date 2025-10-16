mod pdf;
mod types;
mod util;

use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    body::Bytes,
    extract::{FromRequest, Multipart, Path, Request, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::pdf::{
    extract::{extract_document, ExtractedDocument},
    loader::{parse_document, persist_original},
    patch::apply_patches,
};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    extraction: ExtractedDocument,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct JsonPayload {
    file: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::default();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<header::HeaderValue>()?)
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
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    PdfPayload { bytes }: PdfPayload,
) -> Result<Json<OpenResponse>, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::BadRequest("empty payload".into()));
    }

    parse_document(&bytes)?;
    let extraction = extract_document(&bytes)?;
    let doc_id = allocate_id();
    let path = persist_original(&doc_id, &bytes)?;
    let entry = LoadedDoc {
        path,
        bytes: bytes.clone(),
        extraction,
    };

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), entry);
    drop(store);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let entry = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(entry.extraction.ir.clone()))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let entry = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;

    let updated = apply_patches(&entry.bytes, &entry.extraction, &ops)?;
    fs::write(&entry.path, &updated).await?;
    let extraction = extract_document(&updated)?;
    entry.bytes = updated;
    entry.extraction = extraction;

    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&entry.bytes)
    );

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
        ..PatchResponse::default()
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
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

struct PdfPayload {
    bytes: Vec<u8>,
}

#[axum::async_trait]
impl<S> FromRequest<S> for PdfPayload
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let content_type = parts.headers.get(header::CONTENT_TYPE).cloned();
        if let Some(value) = content_type {
            let content = value
                .to_str()
                .map_err(|_| ApiError::BadRequest("invalid content-type".into()))?;
            if content.starts_with("multipart/form-data") {
                let request = Request::from_parts(parts, body);
                let mut multipart = Multipart::from_request(request, state).await?;
                while let Some(field) = multipart.next_field().await? {
                    if field.name() == Some("file") {
                        let data = field.bytes().await?.to_vec();
                        if data.is_empty() {
                            return Err(ApiError::BadRequest("supplied file is empty".into()));
                        }
                        return Ok(PdfPayload { bytes: data });
                    }
                }
                Err(ApiError::BadRequest("file field missing".into()))
            } else if content.starts_with("application/json") {
                let request = Request::from_parts(parts, body);
                let Json(payload) = Json::<JsonPayload>::from_request(request, state).await?;
                let data = decode_base64(&payload.file)?;
                Ok(PdfPayload { bytes: data })
            } else {
                let request = Request::from_parts(parts, body);
                let bytes = Bytes::from_request(request, state).await?;
                Ok(PdfPayload {
                    bytes: bytes.to_vec(),
                })
            }
        } else {
            let request = Request::from_parts(parts, body);
            let bytes = Bytes::from_request(request, state).await?;
            Ok(PdfPayload {
                bytes: bytes.to_vec(),
            })
        }
    }
}

fn decode_base64(input: &str) -> Result<Vec<u8>, ApiError> {
    let data = input
        .split_once(",")
        .map(|(_, tail)| tail)
        .unwrap_or(input)
        .trim();
    if data.is_empty() {
        return Err(ApiError::BadRequest("base64 payload empty".into()));
    }
    BASE64
        .decode(data.as_bytes())
        .map_err(|_| ApiError::BadRequest("invalid base64 payload".into()))
}

fn allocate_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    let idx = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("doc-{idx:04}")
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
    Json(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    Bytes(#[from] axum::extract::rejection::BytesRejection),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Multipart(err) => (
                StatusCode::BAD_REQUEST,
                format!("invalid multipart payload: {err}"),
            )
                .into_response(),
            ApiError::Json(err) => (
                StatusCode::BAD_REQUEST,
                format!("invalid JSON payload: {err}"),
            )
                .into_response(),
            ApiError::Bytes(err) => (
                StatusCode::BAD_REQUEST,
                format!("unable to read body: {err}"),
            )
                .into_response(),
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error whilst processing request");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
