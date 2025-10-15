use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use crate::types::{DocumentIr, PatchOperation};

#[derive(Clone, Default)]
struct AppState {
    inner: Arc<RwLock<HashMap<Uuid, StoredDocument>>>,
}

#[derive(Clone)]
struct StoredDocument {
    pub ir: DocumentIr,
    pub pdf_data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct OpenRequest {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenResponse {
    pub doc_id: Uuid,
}

#[derive(Debug, Serialize)]
struct PatchResponse {
    pub ok: bool,
    #[serde(with = "base64_vec")]
    pub updated_pdf: Vec<u8>,
    pub remap: HashMap<String, serde_json::Value>,
}

mod base64_vec {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = BASE64.encode(bytes);
        serializer.serialize_str(&format!("data:application/pdf;base64,{}", encoded))
    }

    pub fn deserialize<'de, D>(_deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Vec::new())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "pdf_editor_backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    tracing::info!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    Json(_req): Json<OpenRequest>,
) -> Result<Json<OpenResponse>, Response> {
    let doc_id = Uuid::new_v4();
    let ir = pdf::extract::demo_ir();
    let pdf_data = pdf::loader::demo_pdf();

    let mut guard = state.inner.write();
    guard.insert(
        doc_id,
        StoredDocument {
            ir,
            pdf_data,
        },
    );
    drop(guard);

    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<DocumentIr>, Response> {
    let guard = state.inner.read();
    let Some(doc) = guard.get(&doc_id) else {
        return Err(StatusCode::NOT_FOUND.into_response());
    };
    Ok(Json(doc.ir.clone()))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(_ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, Response> {
    let mut guard = state.inner.write();
    let Some(doc) = guard.get_mut(&doc_id) else {
        return Err(StatusCode::NOT_FOUND.into_response());
    };

    // For the scaffold we simply echo the previously stored PDF without modifications.
    let updated_pdf = doc.pdf_data.clone();

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf,
        remap: HashMap::new(),
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Response, Response> {
    let guard = state.inner.read();
    let Some(doc) = guard.get(&doc_id) else {
        return Err(StatusCode::NOT_FOUND.into_response());
    };

    Ok((
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "application/pdf".to_string()),
            (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=doc.pdf".to_string()),
        ],
        doc.pdf_data.clone(),
    )
        .into_response())
}
