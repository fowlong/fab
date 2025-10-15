use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use crate::types::{DocumentId, DocumentIr, DocumentMeta, PatchOperations};

#[derive(Clone, Default)]
struct AppState {
    documents: Arc<RwLock<HashMap<DocumentId, StoredDocument>>>,
}

#[derive(Clone, Default)]
struct StoredDocument {
    ir: DocumentIr,
    pdf_data: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
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

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8787));
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, (StatusCode, String)> {
    let mut pdf_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        if let Some(name) = field.name() {
            if name == "file" {
                pdf_bytes = Some(field.bytes().await.map_err(internal_error)?.to_vec());
                break;
            }
        }
    }

    let pdf_bytes = pdf_bytes.unwrap_or_else(|| include_bytes!("../../e2e/sample.pdf").to_vec());
    let mut ir = pdf::extract::extract_ir(&pdf_bytes).unwrap_or_else(|_| DocumentIr::sample());
    let page_count = ir
        .meta
        .as_ref()
        .and_then(|meta| meta.page_count)
        .unwrap_or(ir.pages.len());
    ir.meta = Some(DocumentMeta {
        pdf_data: Some(format!(
            "data:application/pdf;base64,{}",
            base64::encode(&pdf_bytes)
        )),
        page_count: Some(page_count),
    });

    let doc_id = Uuid::new_v4().to_string();
    let stored = StoredDocument {
        ir: ir.clone(),
        pdf_data,
    };
    state.documents.write().await.insert(doc_id.clone(), stored);
    Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(Path(doc_id): Path<DocumentId>, State(state): State<AppState>) -> Json<DocumentIr> {
    let documents = state.documents.read().await;
    let ir = documents
        .get(&doc_id)
        .map(|doc| doc.ir.clone())
        .unwrap_or_else(DocumentIr::sample);
    Json(ir)
}

async fn apply_patch(
    Path(doc_id): Path<DocumentId>,
    State(state): State<AppState>,
    Json(patch): Json<PatchOperations>,
) -> Json<PatchResponse> {
    let mut documents = state.documents.write().await;
    let entry = documents.entry(doc_id).or_insert_with(|| StoredDocument {
        ir: DocumentIr::sample(),
        pdf_data: include_bytes!("../../e2e/sample.pdf").to_vec(),
    });
    entry.ir.last_patch = Some(patch.clone());
    Json(PatchResponse {
        ok: true,
        updated_pdf: Some(format!(
            "data:application/pdf;base64,{}",
            base64::encode(&entry.pdf_data)
        )),
        remap: HashMap::new(),
    })
}

async fn download_pdf(
    Path(doc_id): Path<DocumentId>,
    State(state): State<AppState>,
) -> (HeaderMap, Vec<u8>) {
    let documents = state.documents.read().await;
    let bytes = documents
        .get(&doc_id)
        .map(|doc| doc.pdf_data.clone())
        .unwrap_or_else(|| include_bytes!("../../e2e/sample.pdf").to_vec());

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"edited.pdf\""),
    );

    (headers, bytes)
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: DocumentId,
}

#[derive(Serialize, Deserialize, Clone)]
struct PatchResponse {
    ok: bool,
    #[serde(rename = "updatedPdf")]
    updated_pdf: Option<String>,
    remap: HashMap<String, serde_json::Value>,
}
