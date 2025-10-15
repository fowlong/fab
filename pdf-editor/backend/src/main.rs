mod pdf;
pub mod types;
pub mod util;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use parking_lot::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::{
    pdf::loader::LoadedDocument,
    types::{DocumentIr, OpenRequest, OpenResponse, PatchOp, PatchResponse},
};

#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    docs: RwLock<HashMap<Uuid, DocumentState>>, // simple in-memory cache for MVP
}

struct DocumentState {
    doc: LoadedDocument,
}

impl AppState {
    fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                docs: RwLock::new(HashMap::new()),
            }),
        }
    }

    fn insert(&self, id: Uuid, doc: LoadedDocument) {
        self.inner.docs.write().insert(id, DocumentState { doc });
    }

    fn with_document<R>(&self, id: &Uuid, f: impl FnOnce(&LoadedDocument) -> R) -> Option<R> {
        self.inner.docs.read().get(id).map(|state| f(&state.doc))
    }

    fn with_document_mut<R>(
        &self,
        id: &Uuid,
        f: impl FnOnce(&mut LoadedDocument) -> anyhow::Result<R>,
    ) -> anyhow::Result<Option<R>> {
        let mut guard = self.inner.docs.write();
        if let Some(state) = guard.get_mut(id) {
            let result = f(&mut state.doc)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let payload = serde_json::json!({
            "ok": false,
            "error": self.to_string(),
        });
        (status, Json(payload)).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let port: u16 = std::env::var("PDF_EDITOR_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8787);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(%addr, "starting pdf editor backend");

    if let Err(err) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        tracing::error!(error = ?err, "server error");
    }
}

async fn open_document(
    State(state): State<AppState>,
    Json(payload): Json<OpenRequest>,
) -> ApiResult<Json<OpenResponse>> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.data.as_bytes())
        .map_err(|err| ApiError::BadRequest(format!("failed to decode base64 data: {err}")))?;

    let mut document = LoadedDocument::from_bytes(bytes)
        .map_err(|err| ApiError::BadRequest(format!("failed to parse pdf: {err}")))?;

    // For the MVP we immediately derive the IR from the loaded document so the
    // first GET /api/ir returns meaningful metadata.
    document
        .refresh_ir()
        .map_err(|err| ApiError::Internal(format!("failed to extract ir: {err}")))?;

    let id = Uuid::new_v4();
    state.insert(id, document);

    Ok(Json(OpenResponse { doc_id: id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> ApiResult<Json<DocumentIr>> {
    state
        .with_document(&doc_id, |doc| doc.ir().clone())
        .ok_or(ApiError::NotFound)
        .map(Json)
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOp>>,
) -> ApiResult<Json<PatchResponse>> {
    let outcome = state
        .with_document_mut(&doc_id, |doc| crate::pdf::patch::apply_ops(doc, &ops))
        .map_err(|err| ApiError::Internal(format!("failed to apply patch: {err}")))?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: outcome.updated_pdf,
        remap: outcome.remap,
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> ApiResult<Response> {
    state
        .with_document(&doc_id, |doc| doc.current_pdf().to_vec())
        .ok_or(ApiError::NotFound)
        .map(|bytes| {
            let mut response = Response::new(bytes.into());
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/pdf"),
            );
            response
        })
}
