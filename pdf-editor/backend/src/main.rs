mod pdf;
mod types;
mod util;

use std::net::SocketAddr;

use axum::{
    extract::Path, extract::State, http::StatusCode, response::IntoResponse, routing::get,
    routing::post, Json, Router,
};
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

use crate::pdf::{extract, patch};
use crate::types::{
    DocId, IrDocument, OpenPdfRequest, OpenPdfResponse, PatchRequest, PatchResponse,
};

#[derive(Clone, Default)]
struct AppState {
    inner: std::sync::Arc<RwLock<DocumentStore>>,
}

#[derive(Default)]
struct DocumentStore {
    documents: std::collections::HashMap<DocId, DocumentRecord>,
}

struct DocumentRecord {
    id: DocId,
    name: Option<String>,
    bytes: Vec<u8>,
    ir: IrDocument,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let state = AppState::default();

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    info!("listening", %addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn setup_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

async fn health() -> &'static str {
    "ok"
}

async fn open_pdf(
    State(state): State<AppState>,
    Json(payload): Json<OpenPdfRequest>,
) -> Result<Json<OpenPdfResponse>, AppError> {
    let pdf_bytes = base64::decode(&payload.pdf_base64)?;
    let ir = extract::extract_document_ir(&pdf_bytes)?;

    let doc_id = DocId::new(Uuid::new_v4());
    let record = DocumentRecord {
        id: doc_id.clone(),
        name: payload.name.clone(),
        bytes: pdf_bytes,
        ir,
    };

    let mut guard = state.inner.write().await;
    guard.documents.insert(doc_id.clone(), record);

    Ok(Json(OpenPdfResponse { doc_id }))
}

async fn get_ir(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<IrDocument>, AppError> {
    let doc_id = DocId::parse(doc_id)?;
    let guard = state.inner.read().await;
    let record = guard
        .documents
        .get(&doc_id)
        .ok_or_else(|| AppError::not_found("document"))?;
    Ok(Json(record.ir.clone()))
}

async fn apply_patch(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(patch_ops): Json<PatchRequest>,
) -> Result<Json<PatchResponse>, AppError> {
    let doc_id = DocId::parse(doc_id)?;
    let mut guard = state.inner.write().await;
    let record = guard
        .documents
        .get_mut(&doc_id)
        .ok_or_else(|| AppError::not_found("document"))?;

    let outcome = patch::apply_patch(&record.bytes, &record.ir, &patch_ops)?;
    record.bytes = outcome.updated_pdf.clone();
    record.ir = outcome.updated_ir;

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(base64::encode(outcome.updated_pdf)),
        remap: outcome.remap,
    }))
}

async fn download_pdf(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let doc_id = DocId::parse(doc_id)?;
    let guard = state.inner.read().await;
    let record = guard
        .documents
        .get(&doc_id)
        .ok_or_else(|| AppError::not_found("document"))?;
    let body = record.bytes.clone();
    Ok((
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/pdf"),
        )],
        body,
    ))
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}

impl AppError {
    fn not_found(entity: &str) -> Self {
        AppError::Message(format!("{entity} not found"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::Message(ref msg) if msg.contains("not found") => StatusCode::NOT_FOUND,
            AppError::Base64(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        error!(error = ?self, "request failed");
        (status, self.to_string()).into_response()
    }
}
