mod pdf;
mod types;
mod util;

use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use pdf::extract::extract_ir;
use pdf::loader::{DocumentEntry, DocumentStore};
use pdf::patch::apply_patches;
use pdf::write::incremental_save;
use serde::Serialize;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::types::{DocumentIR, PatchOp, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    store: Arc<DocumentStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: Arc::new(DocumentStore::new()),
    };

    let app = Router::new()
        .route("/api/open", post(open_handler))
        .route("/api/ir/:doc_id", get(ir_handler))
        .route("/api/patch/:doc_id", post(patch_handler))
        .route("/api/pdf/:doc_id", get(pdf_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8787").await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("received shutdown signal");
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = self.0.to_string();
        let body = Json(serde_json::json!({ "error": message }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

#[derive(Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: String,
}

async fn open_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    let mut pdf_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError(anyhow::Error::new(err)))?
    {
        let name = field.name().map(|s| s.to_string());
        if name.as_deref() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|err| ApiError(anyhow::Error::new(err)))?;
            pdf_bytes = Some(data.to_vec());
            break;
        }
    }

    let pdf_bytes = pdf_bytes.ok_or_else(|| ApiError(anyhow::anyhow!("missing file field")))?;
    let ir = extract_ir(&pdf_bytes).map_err(ApiError)?;
    let entry = DocumentEntry::new(pdf_bytes.clone(), ir.clone());
    let doc_id = entry.id.clone();
    state.store.insert(entry);
    Ok(Json(OpenResponse { doc_id }))
}

async fn ir_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, ApiError> {
    let entry = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError(anyhow::anyhow!("document not found")))?;
    Ok(Json(entry.ir))
}

async fn patch_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let entry = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError(anyhow::anyhow!("document not found")))?;

    let mut ir = entry.ir.clone();
    if let Err(err) = apply_patches(&mut ir, &ops) {
        return Ok(Json(PatchResponse {
            ok: false,
            updated_pdf: None,
            remap: None,
            error: Some(err.to_string()),
        }));
    }

    let updated_pdf = incremental_save(&ir, &entry.current_pdf).map_err(ApiError)?;
    let encoded = base64::encode(&updated_pdf);
    state.store.update_ir(&doc_id, ir);
    state.store.update_pdf(&doc_id, updated_pdf.clone());
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(format!("data:application/pdf;base64,{}", encoded)),
        remap: None,
        error: None,
    }))
}

async fn pdf_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, ApiError> {
    let entry = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError(anyhow::anyhow!("document not found")))?;
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        entry.current_pdf,
    )
        .into_response())
}
