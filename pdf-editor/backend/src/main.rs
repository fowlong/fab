use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::extract::Multipart;
use axum::{
    extract::Path,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;
use tracing::{error, info};

mod pdf;
mod types;
mod util;

use types::{DocumentIr, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    documents: Arc<RwLock<HashMap<String, StoredDocument>>>,
}

struct StoredDocument {
    bytes: Vec<u8>,
    ir: Option<DocumentIr>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState::default();
    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("server error");
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        error!(?err, "failed to read multipart field");
        StatusCode::BAD_REQUEST
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|err| {
                error!(?err, "failed to read uploaded file");
                StatusCode::BAD_REQUEST
            })?;
            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or(StatusCode::BAD_REQUEST)?;
    let doc_id = uuid::Uuid::new_v4().to_string();

    let mut documents = state.documents.write().await;
    documents.insert(doc_id.clone(), StoredDocument { bytes, ir: None });

    Ok(Json(serde_json::json!({ "docId": doc_id })))
}

async fn fetch_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let documents = state.documents.read().await;
    let doc = documents.get(&doc_id).ok_or(StatusCode::NOT_FOUND)?;

    let ir = doc.ir.clone().unwrap_or_else(|| DocumentIr {
        pages: vec![],
        source_pdf_url: None,
    });

    Ok(Json(ir))
}

async fn apply_patch(
    State(_state): State<AppState>,
    Path(_doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(?ops, "received patch operations");
    let response = PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    };
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let documents = state.documents.read().await;
    let doc = documents.get(&doc_id).ok_or(StatusCode::NOT_FOUND)?;
    let bytes = doc.bytes.clone();
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        bytes,
    ))
}
