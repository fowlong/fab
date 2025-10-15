mod pdf;
mod types;
mod util;

use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, Method, Request, StatusCode},
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

use crate::pdf::{extract, loader, patch};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: DocStore,
}

#[derive(Clone)]
struct DocStore {
    inner: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: Document,
    ir_cache: Option<DocumentIR>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenJsonRequest {
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
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let state = AppState {
        store: DocStore::new(),
    };

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
    mut request: Request<Body>,
) -> Result<Json<OpenResponse>, ApiError> {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();

    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &state).await?;
        let mut pdf_bytes: Option<Vec<u8>> = None;
        while let Some(field) = multipart.next_field().await? {
            if field.name() == Some("file") {
                pdf_bytes = Some(field.bytes().await?.to_vec());
                break;
            }
        }
        let bytes = pdf_bytes.ok_or_else(|| ApiError::BadRequest("file field missing".into()))?;
        let doc_id = state.store.open(bytes).await?;
        Ok(Json(OpenResponse { doc_id }))
    } else {
        let (parts, body) = request.into_parts();
        let payload_bytes = hyper::body::to_bytes(body).await?;
        drop(parts);
        if content_type.starts_with("application/json") || content_type.is_empty() {
            let payload: OpenJsonRequest = serde_json::from_slice(&payload_bytes)?;
            let encoded = payload
                .data
                .strip_prefix("data:application/pdf;base64,")
                .unwrap_or(&payload.data);
            let bytes = BASE64
                .decode(encoded)
                .map_err(|_| ApiError::InvalidBase64)?;
            let doc_id = state.store.open(bytes).await?;
            Ok(Json(OpenResponse { doc_id }))
        } else {
            Err(ApiError::UnsupportedMediaType)
        }
    }
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let ir = state.store.ir(&doc_id).await?;
    Ok(Json(ir))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let ir = state.store.ir(&doc_id).await?;
    let outcome = state.store.apply(&doc_id, &ir, &ops).await?;
    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&outcome.bytes)
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
    let bytes = state.store.bytes(&doc_id).await?;
    let mut response = Response::new(bytes.into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

impl DocStore {
    fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn open(&self, bytes: Vec<u8>) -> Result<String, ApiError> {
        if bytes.is_empty() {
            return Err(ApiError::BadRequest("empty PDF payload".into()));
        }
        let doc_id = new_doc_id();
        let dir = PathBuf::from("/tmp/fab");
        fs::create_dir_all(&dir).await?;
        let path = dir.join(format!("{doc_id}.pdf"));
        fs::write(&path, &bytes).await?;
        let parse_bytes = bytes.clone();
        let document = tokio::task::spawn_blocking(move || loader::parse_document(&parse_bytes))
            .await
            .map_err(|err| ApiError::Internal(err.into()))??;
        let mut inner = self.inner.write().await;
        inner.insert(
            doc_id.clone(),
            LoadedDoc {
                path,
                bytes,
                document,
                ir_cache: None,
            },
        );
        Ok(doc_id)
    }

    async fn ir(&self, doc_id: &str) -> Result<DocumentIR, ApiError> {
        if let Some(cached) = self.cached_ir(doc_id).await? {
            return Ok(cached);
        }
        let document = {
            let inner = self.inner.read().await;
            let entry = inner.get(doc_id).ok_or(ApiError::NotFound)?;
            entry.document.clone()
        };
        let ir = tokio::task::spawn_blocking(move || extract::extract_ir(&document))
            .await
            .map_err(|err| ApiError::Internal(err.into()))??;
        let mut inner = self.inner.write().await;
        if let Some(entry) = inner.get_mut(doc_id) {
            entry.ir_cache = Some(ir.clone());
        }
        Ok(ir)
    }

    async fn cached_ir(&self, doc_id: &str) -> Result<Option<DocumentIR>, ApiError> {
        let inner = self.inner.read().await;
        let entry = match inner.get(doc_id) {
            Some(entry) => entry,
            None => return Ok(None),
        };
        Ok(entry.ir_cache.clone())
    }

    async fn apply(
        &self,
        doc_id: &str,
        ir: &DocumentIR,
        ops: &[PatchOperation],
    ) -> Result<patch::PatchOutcome, ApiError> {
        let (document, bytes) = {
            let inner = self.inner.read().await;
            let entry = inner.get(doc_id).ok_or(ApiError::NotFound)?;
            (entry.document.clone(), entry.bytes.clone())
        };
        let ir_clone = ir.clone();
        let ops_vec = ops.to_vec();
        let outcome = tokio::task::spawn_blocking(move || {
            patch::apply_patches(&document, &bytes, &ir_clone, &ops_vec)
        })
        .await
        .map_err(|err| ApiError::Internal(err.into()))??;

        let (path, bytes_to_write) = {
            let mut inner = self.inner.write().await;
            let entry = inner.get_mut(doc_id).ok_or(ApiError::NotFound)?;
            entry.bytes = outcome.bytes.clone();
            entry.document = outcome.document.clone();
            entry.ir_cache = Some(outcome.ir.clone());
            (entry.path.clone(), entry.bytes.clone())
        };
        fs::write(path, &bytes_to_write).await?;
        Ok(outcome)
    }

    async fn bytes(&self, doc_id: &str) -> Result<Vec<u8>, ApiError> {
        let inner = self.inner.read().await;
        let entry = inner.get(doc_id).ok_or(ApiError::NotFound)?;
        Ok(entry.bytes.clone())
    }
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("unsupported media type")]
    UnsupportedMediaType,
    #[error("{0}")]
    BadRequest(String),
    #[error("invalid base64 payload")]
    InvalidBase64,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Body(#[from] hyper::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::Multipart(_) | ApiError::BadRequest(_) | ApiError::InvalidBase64 => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            ApiError::UnsupportedMediaType => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()).into_response()
            }
            ApiError::Json(_) | ApiError::Body(_) | ApiError::Internal(_) => {
                tracing::error!(error = %self, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

fn new_doc_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}
