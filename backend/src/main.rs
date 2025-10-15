mod pdf;
mod types;
mod util;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{
    async_trait,
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hyper::body::to_bytes;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pdf::loader::{persist_temp, LoadedDoc};
use crate::types::{DocumentIR, PatchOperation, PatchResponse};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<HashMap<String, LoadedDoc>>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
}

#[derive(serde::Deserialize)]
struct OpenJsonPayload {
    data: String,
}

struct OpenPayload(Vec<u8>);

#[async_trait]
impl<S> axum::extract::FromRequest<S> for OpenPayload
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_default();

        if content_type.contains("application/json") {
            let bytes = to_bytes(req.into_body())
                .await
                .map_err(|err| ApiError::Internal(err.into()))?;
            let payload: OpenJsonPayload = serde_json::from_slice(&bytes)
                .map_err(|err| ApiError::BadRequest(format!("invalid json payload: {err}")))?;
            let data = BASE64
                .decode(payload.data.as_bytes())
                .map_err(|err| ApiError::BadRequest(format!("invalid base64 payload: {err}")))?;
            if data.is_empty() {
                return Err(ApiError::BadRequest("empty PDF payload".into()));
            }
            return Ok(OpenPayload(data));
        }

        let mut multipart = Multipart::from_request(req, state)
            .await
            .map_err(ApiError::Multipart)?;
        while let Some(field) = multipart.next_field().await.map_err(ApiError::Multipart)? {
            if field.name() == Some("file") {
                let data = field.bytes().await.map_err(ApiError::Multipart)?.to_vec();
                if data.is_empty() {
                    return Err(ApiError::BadRequest("empty PDF payload".into()));
                }
                return Ok(OpenPayload(data));
            }
        }

        Err(ApiError::BadRequest("missing file field".into()))
    }
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
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
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
    OpenPayload(bytes): OpenPayload,
) -> Result<Json<OpenResponse>, ApiError> {
    let doc_id = new_doc_id();
    let path = persist_temp(&doc_id, &bytes).map_err(ApiError::Internal)?;
    let mut loaded = LoadedDoc::new(bytes, path).map_err(ApiError::Internal)?;
    // Pre-cache IR for page 0
    let _ = loaded.ensure_page0().map_err(ApiError::Internal)?;

    let mut store = state.store.write().await;
    store.insert(doc_id.clone(), loaded);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let extraction = doc.ensure_page0().map_err(ApiError::Internal)?;
    Ok(Json(DocumentIR {
        pages: vec![extraction.ir.clone()],
    }))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut store = state.store.write().await;
    let doc = store.get_mut(&doc_id).ok_or(ApiError::NotFound)?;
    let outcome = pdf::patch::apply_patches(doc, &ops).map_err(ApiError::Internal)?;
    let encoded = format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(&outcome.updated_bytes)
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
