mod pdf;
mod types;
mod util;

use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{self, header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::either::Either;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{extract::PageCache, loader, patch};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<DocStore>>,
}

struct DocStore {
    docs: HashMap<String, LoadedDoc>,
    counter: usize,
}

struct LoadedDoc {
    path: PathBuf,
    bytes: Vec<u8>,
    document: lopdf::Document,
    page_cache: Option<PageCache>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenJsonPayload {
    data: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: Arc::new(RwLock::new(DocStore::default())),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([http::Method::GET, http::Method::POST])
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
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    payload: Either<Multipart, Json<OpenJsonPayload>>,
) -> Result<Json<OpenResponse>, ApiError> {
    let pdf_bytes = match payload {
        Either::E1(mut multipart) => read_multipart_pdf(&mut multipart).await?,
        Either::E2(Json(body)) => decode_base64_payload(body.data).await?,
    };

    let mut store = state.store.write().await;
    let doc_id = store.next_id();
    let path = PathBuf::from(format!("/tmp/fab/{doc_id}.pdf"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&path, &pdf_bytes).await?;

    let document = loader::parse_document(&pdf_bytes)?;
    let extraction = pdf::extract::extract_page_zero(&document)?;

    store.docs.insert(
        doc_id.clone(),
        LoadedDoc {
            path,
            bytes: pdf_bytes,
            document,
            page_cache: Some(extraction.page_cache),
        },
    );

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if doc.page_cache.is_none() {
        let extraction = pdf::extract::extract_page_zero(&doc.document)?;
        doc.page_cache = Some(extraction.page_cache);
    }
    let page_ir = doc
        .page_cache
        .as_ref()
        .map(|cache| cache.page_ir.clone())
        .context("page cache missing")?;
    Ok(Json(DocumentIR {
        pages: vec![page_ir],
    }))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.docs.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    if doc.page_cache.is_none() {
        let extraction = pdf::extract::extract_page_zero(&doc.document)?;
        doc.page_cache = Some(extraction.page_cache);
    }

    let outcome = patch::apply_patches(
        &doc.document,
        &doc.bytes,
        doc.page_cache.as_ref().context("page cache unavailable")?,
        &ops,
    )?;

    if let Some(outcome) = outcome {
        doc.bytes = outcome.bytes;
        doc.document = outcome.document;
        doc.page_cache = Some(outcome.page_cache);
        fs::write(&doc.path, &doc.bytes).await?;
    }

    let encoded = format!("data:application/pdf;base64,{}", BASE64.encode(&doc.bytes));

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
    let doc = store.docs.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(Bytes::from(doc.bytes.clone()));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn read_multipart_pdf(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(ApiError::BadRequest("missing file field".into()))
}

async fn decode_base64_payload(data: Option<String>) -> Result<Vec<u8>, ApiError> {
    let data = data.ok_or_else(|| ApiError::BadRequest("missing data payload".into()))?;
    let encoded = data
        .split_once(',')
        .map(|(_, tail)| tail.to_string())
        .unwrap_or(data);
    BASE64
        .decode(encoded.as_bytes())
        .map_err(|err| ApiError::BadRequest(format!("invalid base64 payload: {err}")))
}

impl Default for DocStore {
    fn default() -> Self {
        Self {
            docs: HashMap::new(),
            counter: 0,
        }
    }
}

impl DocStore {
    fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("doc-{:04}", self.counter)
    }
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
