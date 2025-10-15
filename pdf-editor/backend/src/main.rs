mod pdf;
mod types;
mod util;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::extract::Multipart;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::pdf::extract;
use crate::types::{DocumentIr, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    docs: Arc<RwLock<HashMap<String, DocumentEntry>>>,
}

struct DocumentEntry {
    bytes: Vec<u8>,
    ir: DocumentIr,
}

#[derive(thiserror::Error, Debug)]
enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<ApiError> for (StatusCode, String) {
    fn from(err: ApiError) -> Self {
        match &err {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse().unwrap();
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        if let Some(name) = field.name() {
            if name == "file" {
                let data = field.bytes().await.map_err(internal_error)?;
                file_bytes = Some(data.to_vec());
                break;
            }
        }
    }
    let bytes = file_bytes.ok_or_else(|| ApiError::BadRequest("missing file field".into()))?;
    let ir = extract::build_ir(&bytes).map_err(internal_error)?;

    let doc_id = uuid::Uuid::new_v4().to_string();
    let entry = DocumentEntry {
        bytes,
        ir: ir.clone(),
    };
    state.docs.write().await.insert(doc_id.clone(), entry);

    Ok(Json(serde_json::json!({ "docId": doc_id, "ir": ir })))
}

async fn get_ir(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Json<DocumentIr>, (StatusCode, String)> {
    let docs = state.docs.read().await;
    let entry = docs
        .get(&doc_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "document not found".to_string()))?;
    Ok(Json(entry.ir.clone()))
}

async fn apply_patch(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, (StatusCode, String)> {
    let mut docs = state.docs.write().await;
    let entry = docs
        .get_mut(&doc_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "document not found".to_string()))?;

    if let Err(err) = crate::pdf::patch::apply_patch(&mut entry.ir, &ops) {
        error!(?err, "failed to apply patch");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let updated_pdf = base64::encode(&entry.bytes);
    let response = PatchResponse {
        ok: true,
        updated_pdf: Some(format!("data:application/pdf;base64,{}", updated_pdf)),
        remap: None,
        error: None,
    };
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let docs = state.docs.read().await;
    let entry = docs
        .get(&doc_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "document not found".to_string()))?;
    let body = axum::body::Body::from(entry.bytes.clone());
    let mut response = axum::response::Response::new(body);
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/pdf"),
    );
    *response.status_mut() = StatusCode::OK;
    Ok(response)
}

fn internal_error<E: Into<anyhow::Error>>(err: E) -> (StatusCode, String) {
    let err = err.into();
    error!(?err, "request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
