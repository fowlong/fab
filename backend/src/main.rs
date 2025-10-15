use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use crate::pdf::{extract, loader, patch};
use crate::types::{DocumentIR, OpenResponse, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    store: Arc<DocStore>,
}

#[derive(Default)]
struct DocStore {
    docs: RwLock<HashMap<String, StoredDoc>>,
}

#[derive(Clone)]
struct StoredDoc {
    bytes: Vec<u8>,
    ir: DocumentIR,
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
        store: Arc::new(DocStore::default()),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse().expect("valid listen addr");
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening", %addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut file_name = "document.pdf".to_string();
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(ApiError::from)? {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                file_name = name.to_string();
            }
            let data = field.bytes().await.map_err(ApiError::from)?;
            bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = bytes.ok_or_else(|| ApiError::bad_request("Missing file field"))?;
    let mut ir = loader::build_initial_ir(&bytes, &file_name).map_err(ApiError::from)?;
    ir = extract::extract_ir(ir).map_err(ApiError::from)?;

    let doc = StoredDoc { bytes, ir };
    let doc_id = state.store.insert(doc);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DocumentIR>, ApiError> {
    let ir = state
        .store
        .with_doc(&doc_id, |doc| doc.ir.clone())
        .ok_or_else(|| ApiError::not_found(&doc_id))?;
    Ok(Json(ir))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(operations): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let outcome = state.store.try_with_doc_mut(&doc_id, |doc| {
        let patch_outcome = patch::apply_patches(&mut doc.ir, &doc.bytes, &operations)?;
        if let Some(bytes) = patch_outcome.updated_bytes {
            doc.bytes = bytes;
        }
        Ok::<_, anyhow::Error>(patch_outcome.response)
    })?;

    Ok(Json(outcome))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let bytes = state
        .store
        .with_doc(&doc_id, |doc| doc.bytes.clone())
        .ok_or_else(|| ApiError::not_found(&doc_id))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.pdf\"", doc_id),
        )
        .body(axum::body::Full::from(bytes))
        .unwrap();

    Ok(response)
}

impl DocStore {
    fn insert(&self, doc: StoredDoc) -> String {
        let id = Uuid::new_v4().to_string();
        self.docs.write().insert(id.clone(), doc);
        id
    }

    fn with_doc<R>(&self, id: &str, f: impl FnOnce(&StoredDoc) -> R) -> Option<R> {
        let guard = self.docs.read();
        guard.get(id).map(f)
    }

    fn try_with_doc_mut<F, R>(&self, id: &str, f: F) -> Result<R, ApiError>
    where
        F: FnOnce(&mut StoredDoc) -> Result<R, anyhow::Error>,
    {
        let mut guard = self.docs.write();
        let doc = guard.get_mut(id).ok_or_else(|| ApiError::not_found(id))?;
        f(doc).map_err(ApiError::from)
    }
}

struct ApiError {
    status: StatusCode,
    message: String,
    source: Option<anyhow::Error>,
}

impl ApiError {
    fn not_found(id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: format!("Document {id} not found"),
            source: None,
        }
    }

    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
            source: None,
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal server error".to_string(),
            source: Some(error),
        }
    }
}

impl From<axum::extract::multipart::MultipartError> for ApiError {
    fn from(error: axum::extract::multipart::MultipartError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid multipart payload".to_string(),
            source: Some(anyhow::Error::from(error)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Some(source) = &self.source {
            tracing::error!(error = ?source, "API error: {}", self.message);
        }
        let payload = serde_json::json!({
            "ok": false,
            "message": self.message,
        });
        let status = self.status;
        (status, Json(payload)).into_response()
    }
}
