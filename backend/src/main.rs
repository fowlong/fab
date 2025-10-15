mod pdf;
mod types;
mod util;

use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{
    body::Body,
    extract::{FromRequest, Multipart, Path as AxumPath, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{
    extract::{extract_page_zero, rebuild_page, PageExtraction},
    loader,
    patch::{apply_patches, combine_streams},
    write,
};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<DocStore>>,
}

#[derive(Default)]
struct DocStore {
    docs: HashMap<String, LoadedDoc>,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: lopdf::Document,
    page0: Option<PageExtraction>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, Deserialize)]
struct Base64Body {
    file: String,
}

enum OpenPayload {
    Bytes(Vec<u8>),
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
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
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
    payload: OpenPayload,
) -> Result<Json<OpenResponse>, ApiError> {
    let bytes = match payload {
        OpenPayload::Bytes(data) if !data.is_empty() => data,
        _ => return Err(ApiError::InvalidPayload("empty payload".into())),
    };

    let document = loader::parse_document(&bytes)?;
    let doc_id = new_doc_id();
    let path = persist_pdf(&doc_id, &bytes)?;

    let mut store = state.store.write().await;
    store.docs.insert(
        doc_id.clone(),
        LoadedDoc {
            path,
            bytes,
            document,
            page0: None,
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if doc.page0.is_none() {
        let extraction = extract_page_zero(&doc.document)?;
        doc.page0 = Some(extraction.page);
        return Ok(Json(extraction.document_ir));
    }
    let page = doc.page0.as_ref().unwrap();
    Ok(Json(DocumentIR {
        pages: vec![page.ir.clone()],
    }))
}

async fn apply_patch(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if doc.page0.is_none() {
        let extraction = extract_page_zero(&doc.document)?;
        doc.page0 = Some(extraction.page);
    }
    let page = doc.page0.as_mut().unwrap();
    let modified = apply_patches(page, &ops, &doc.document)?;
    if !modified {
        return Ok(Json(PatchResponse {
            ok: true,
            updated_pdf: Some(encode_data_url(&doc.bytes)),
            remap: None,
        }));
    }

    let combined = combine_streams(&page.streams);
    let updated_bytes =
        write::incremental_update(&doc.bytes, &doc.document, page.page_id, combined)?;
    doc.bytes = updated_bytes;
    doc.document = loader::parse_document(&doc.bytes)?;
    doc.page0 = None;
    persist_existing(&doc.path, &doc.bytes)?;

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encode_data_url(&doc.bytes)),
        remap: None,
    }))
}

async fn download_pdf(
    AxumPath(doc_id): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let store = state.store.read().await;
    let doc = store.docs.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(doc.bytes.clone().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

fn persist_pdf(doc_id: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let dir = Path::new("/tmp/fab");
    fs::create_dir_all(dir).map_err(|err| ApiError::Internal(err.into()))?;
    let path = dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, bytes).map_err(|err| ApiError::Internal(err.into()))?;
    Ok(path)
}

fn persist_existing(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    fs::write(path, bytes).map_err(|err| ApiError::Internal(err.into()))
}

fn encode_data_url(bytes: &[u8]) -> String {
    format!("data:application/pdf;base64,{}", BASE64.encode(bytes))
}

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("doc-{id:04}")
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("unsupported content type")]
    UnsupportedContentType,
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "document not found").into_response(),
            ApiError::InvalidPayload(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::UnsupportedContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported content type",
            )
                .into_response(),
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

#[axum::async_trait]
impl<S> FromRequest<S> for OpenPayload
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned());

        if let Some(ct) = content_type {
            if ct.starts_with("multipart/form-data") {
                let mut multipart = Multipart::from_request(req, state).await?;
                while let Some(field) = multipart.next_field().await? {
                    if field.name() == Some("file") {
                        let data = field.bytes().await?.to_vec();
                        if data.is_empty() {
                            continue;
                        }
                        return Ok(OpenPayload::Bytes(data));
                    }
                }
                return Err(ApiError::InvalidPayload(
                    "multipart payload missing file".into(),
                ));
            }
            if ct.starts_with("application/json") {
                let bytes = hyper::body::to_bytes(req.into_body())
                    .await
                    .map_err(|err| ApiError::Internal(err.into()))?;
                let payload: Base64Body = serde_json::from_slice(&bytes)
                    .map_err(|err| ApiError::InvalidPayload(err.to_string()))?;
                let decoded = decode_base64_pdf(&payload.file)?;
                return Ok(OpenPayload::Bytes(decoded));
            }
            if ct == "application/pdf" {
                let bytes = hyper::body::to_bytes(req.into_body())
                    .await
                    .map_err(|err| ApiError::Internal(err.into()))?;
                return Ok(OpenPayload::Bytes(bytes.to_vec()));
            }
            return Err(ApiError::UnsupportedContentType);
        }

        let bytes = hyper::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(err.into()))?;
        Ok(OpenPayload::Bytes(bytes.to_vec()))
    }
}

fn decode_base64_pdf(data: &str) -> Result<Vec<u8>, ApiError> {
    let payload = data.split_once(',').map(|(_, body)| body).unwrap_or(data);
    BASE64
        .decode(payload.trim())
        .map_err(|err| ApiError::InvalidPayload(err.to_string()))
}
