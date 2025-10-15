use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose};
use parking_lot::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::{extract::extract_ir, patch::apply_patch_operations, write::PdfUpdate};
use types::{DocumentIr, PatchOperation};

#[derive(Clone, Default)]
struct AppState {
    docs: Arc<RwLock<HashMap<Uuid, StoredDocument>>>,
}

struct StoredDocument {
    original: Vec<u8>,
    latest: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    tracing::info!(%addr, "starting server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn init_tracing() {
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer());
    subscriber.init();
}

async fn open_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut file_bytes = None;
    while let Some(field) = multipart.next_field().await.transpose()? {
        if field.name() == Some("file") {
            let data = field.bytes().await?.to_vec();
            file_bytes = Some(data);
            break;
        }
    }

    let bytes = file_bytes.ok_or(ApiError::bad_request("file field missing"))?;
    let id = Uuid::new_v4();
    let doc = StoredDocument {
        original: bytes.clone(),
        latest: bytes,
    };
    state.docs.write().insert(id, doc);
    Ok(Json(OpenResponse { doc_id: id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<DocumentIr>, ApiError> {
    let docs = state.docs.read();
    let doc = docs
        .get(&doc_id)
        .ok_or(ApiError::not_found("document not found"))?;
    let ir = extract_ir(&doc.latest)?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut docs = state.docs.write();
    let doc = docs
        .get_mut(&doc_id)
        .ok_or(ApiError::not_found("document not found"))?;
    let PdfUpdate { updated_pdf, remap } = apply_patch_operations(&doc.latest, &ops)?;
    doc.latest = updated_pdf.clone();
    let data_url = format!(
        "data:application/pdf;base64,{}",
        general_purpose::STANDARD.encode(&updated_pdf)
    );
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(data_url),
        remap,
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let docs = state.docs.read();
    let doc = docs
        .get(&doc_id)
        .ok_or(ApiError::not_found("document not found"))?;
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/pdf")
        .body(doc.latest.clone().into())
        .unwrap();
    Ok(resp)
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        tracing::error!(error = %value, "internal error");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

#[derive(serde::Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: Uuid,
}

#[derive(serde::Serialize)]
struct PatchResponse {
    ok: bool,
    #[serde(rename = "updatedPdf")]
    updated_pdf: Option<String>,
    remap: Option<HashMap<String, types::ObjectRemap>>,
}
