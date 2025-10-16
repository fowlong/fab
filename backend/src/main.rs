mod pdf;
mod types;

use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}, sync::Arc};

use anyhow::Result as AnyResult;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header::{ACCEPT, CONTENT_TYPE}, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::extract::multipart::Multipart;
use base64::Engine;
use tokio::sync::RwLock;
use tokio::{fs, net::TcpListener};
use tower_http::cors::CorsLayer;

use crate::pdf::{extract, loader, patch, write};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Clone)]
struct AppState {
    store: Arc<DocStore>,
}

#[derive(Default)]
struct DocStore {
    inner: RwLock<HashMap<String, LoadedDoc>>,
}

struct LoadedDoc {
    path: PathBuf,
    original_bytes: Vec<u8>,
    current_bytes: Vec<u8>,
    document: lopdf::Document,
    cache: Option<extract::PageCache>,
    next_object_id: u32,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .layer(cors)
        .with_state(AppState {
            store: Arc::new(DocStore::default()),
        });

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<OpenResponse>> {
    let bytes = parse_payload(headers, body).await?;
    let doc_id = state.store.open(bytes).await?;
    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> ApiResult<Json<DocumentIR>> {
    let ir = state.store.get_ir(&doc_id).await?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> ApiResult<Json<PatchResponse>> {
    let bytes = state.store.apply_patches(&doc_id, &ops).await?;
    let encoded = format!(
        "data:application/pdf;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    );
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: None,
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> ApiResult<Response> {
    let bytes = state.store.bytes(&doc_id).await?;
    let mut response = Response::new(bytes.into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn parse_payload(headers: HeaderMap, body: axum::body::Bytes) -> ApiResult<Vec<u8>> {
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        let content_type = content_type.to_str().map_err(|_| ApiError::InvalidPayload)?;
        if content_type.starts_with("multipart/form-data") {
            let mut multipart = Multipart::new(headers.clone(), Body::from(body));
            while let Some(field) = multipart.next_field().await.map_err(ApiError::from)? {
                if field.name() == Some("file") {
                    return Ok(field.bytes().await.map_err(ApiError::from)?.to_vec());
                }
            }
            return Err(ApiError::InvalidPayload);
        } else if content_type.starts_with("application/json") {
            #[derive(serde::Deserialize)]
            struct OpenJson {
                data: String,
            }
            let payload: OpenJson = serde_json::from_slice(&body).map_err(ApiError::from)?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload.data.trim())
                .map_err(|_| ApiError::InvalidPayload)?;
            return Ok(bytes);
        }
    }
    Err(ApiError::InvalidPayload)
}

impl DocStore {
    async fn open(&self, bytes: Vec<u8>) -> ApiResult<String> {
        let mut document = loader::parse_document(&bytes).map_err(ApiError::from)?;
        let cache = extract::extract_page_zero(&document).map_err(ApiError::from)?;
        let doc_id = new_doc_id();
        let path = PathBuf::from(format!("/tmp/fab/{doc_id}.pdf"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(ApiError::from)?;
        }
        fs::write(&path, &bytes).await.map_err(ApiError::from)?;
        let next_object_id = loader::next_object_id(&document);
        let mut guard = self.inner.write().await;
        guard.insert(
            doc_id.clone(),
            LoadedDoc {
                path,
                original_bytes: bytes.clone(),
                current_bytes: bytes,
                document,
                cache: Some(cache),
                next_object_id,
            },
        );
        Ok(doc_id)
    }

    async fn get_ir(&self, doc_id: &str) -> ApiResult<DocumentIR> {
        let mut guard = self.inner.write().await;
        let entry = guard.get_mut(doc_id).ok_or(ApiError::NotFound)?;
        if entry.cache.is_none() {
            entry.cache = Some(extract::extract_page_zero(&entry.document).map_err(ApiError::from)?);
        }
        Ok(entry
            .cache
            .as_ref()
            .map(|cache| cache.ir.clone())
            .ok_or(ApiError::NotFound)?)
    }

    async fn apply_patches(
        &self,
        doc_id: &str,
        ops: &[PatchOperation],
    ) -> ApiResult<Vec<u8>> {
        let mut guard = self.inner.write().await;
        let entry = guard.get_mut(doc_id).ok_or(ApiError::NotFound)?;
        if ops.is_empty() {
            return Ok(entry.current_bytes.clone());
        }
        for op in ops {
            let cache = match entry.cache.clone() {
                Some(cache) => cache,
                None => extract::extract_page_zero(&entry.document).map_err(ApiError::from)?,
            };
            let plan = patch::apply_transform(&mut entry.document, &cache, op, entry.next_object_id)
                .map_err(ApiError::from)?;
            let write_result = write::incremental_update(
                &entry.current_bytes,
                &entry.document.trailer,
                &plan.updates,
            )
            .map_err(ApiError::from)?;
            entry.current_bytes = write_result.bytes;
            entry.document.trailer = write_result.trailer;
            entry.next_object_id = plan.next_object_id;
            entry.cache = Some(extract::extract_page_zero(&entry.document).map_err(ApiError::from)?);
        }
        fs::write(&entry.path, &entry.current_bytes)
            .await
            .map_err(ApiError::from)?;
        Ok(entry.current_bytes.clone())
    }

    async fn bytes(&self, doc_id: &str) -> ApiResult<Vec<u8>> {
        let guard = self.inner.read().await;
        let entry = guard.get(doc_id).ok_or(ApiError::NotFound)?;
        Ok(entry.current_bytes.clone())
    }
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("invalid payload")]
    InvalidPayload,
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::Internal(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::InvalidPayload => {
                (StatusCode::BAD_REQUEST, "unsupported payload").into_response()
            }
            ApiError::Multipart(_) => {
                (StatusCode::BAD_REQUEST, "malformed multipart payload").into_response()
            }
            ApiError::Internal(err) => {
                eprintln!("internal error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
        }
    }
}

fn new_doc_id() -> String {
    let value = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{value:04}")
}
