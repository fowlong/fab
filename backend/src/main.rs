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
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::{
    loader::{self, LoadedDoc},
    patch,
};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenJsonPayload {
    #[serde(alias = "pdfBase64", alias = "data")]
    pdf_base64: String,
}

static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

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
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

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
    mut req: Request<Body>,
) -> Result<Json<OpenResponse>, ApiError> {
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let bytes = if content_type.contains("multipart/form-data") {
        let multipart = Multipart::from_request(req, &state).await?;
        read_multipart_bytes(multipart).await?
    } else {
        let body = axum::body::to_bytes(req.into_body())
            .await
            .map_err(|err| ApiError::Internal(anyhow::anyhow!(err)))?;
        decode_json_base64(&body)?
    };

    if bytes.is_empty() {
        return Err(ApiError::BadRequest("PDF payload was empty".into()));
    }

    let doc_id = new_doc_id();
    let dir = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&dir).await?;
    let path = dir.join(format!("{doc_id}.pdf"));
    fs::write(&path, &bytes).await?;
    let loaded = loader::load(bytes, path)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), loaded);

    Ok(Json(OpenResponse { doc_id }))
}

async fn read_multipart_bytes(mut multipart: Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(ApiError::BadRequest(
        "multipart payload missing file field".into(),
    ))
}

fn decode_json_base64(body: &[u8]) -> Result<Vec<u8>, ApiError> {
    let payload: OpenJsonPayload = serde_json::from_slice(body)
        .map_err(|err| ApiError::BadRequest(format!("invalid JSON payload: {err}")))?;
    let encoded = payload
        .pdf_base64
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(&payload.pdf_base64);
    BASE64
        .decode(encoded.trim())
        .map_err(|err| ApiError::BadRequest(format!("invalid base64 data: {err}")))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let store = state.store.read().await;
    let doc = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    Ok(Json(doc.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    patch::apply_patches(doc, &ops)?;
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
    let doc = store.get(&doc_id).ok_or(ApiError::NotFound)?;
    let mut response = Response::new(doc.bytes.clone().into());
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

fn new_doc_id() -> String {
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("doc-{id:04}")
}
