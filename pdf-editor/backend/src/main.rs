use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod pdf;
mod types;
mod util;

use types::{IrDocument, PatchOperation};

#[derive(Clone, Default)]
struct AppState {
    documents: Arc<RwLock<Vec<StoredDocument>>>,
}

#[derive(Clone, Default)]
struct StoredDocument {
    id: String,
    _bytes: Vec<u8>,
    ir: IrDocument,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn open_document(State(state): State<AppState>) -> Result<Json<OpenResponse>, AppError> {
    let mut documents = state.documents.write().await;
    let id = format!("doc-{}", documents.len() + 1);
    let ir = IrDocument { pages: Vec::new() };
    documents.push(StoredDocument {
        id: id.clone(),
        _bytes: Vec::new(),
        ir,
    });
    Ok(Json(OpenResponse { doc_id: id }))
}

async fn get_ir(State(state): State<AppState>, axum::extract::Path(doc_id): axum::extract::Path<String>) -> Result<Json<IrDocument>, AppError> {
    let documents = state.documents.read().await;
    let doc = documents
        .iter()
        .find(|doc| doc.id == doc_id)
        .ok_or(AppError::NotFound)?;
    Ok(Json(doc.ir.clone()))
}

async fn apply_patch(
    State(_state): State<AppState>,
    axum::extract::Path(_doc_id): axum::extract::Path<String>,
    Json(_ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, AppError> {
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: serde_json::Value::Object(serde_json::Map::new()),
    }))
}

async fn download_pdf(
    State(_state): State<AppState>,
    axum::extract::Path(_doc_id): axum::extract::Path<String>,
) -> Result<Vec<u8>, AppError> {
    Ok(Vec::new())
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PatchResponse {
    ok: bool,
    #[serde(rename = "updatedPdf")]
    updated_pdf: Option<String>,
    remap: serde_json::Value,
}

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("document not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Other(err) => {
                let body = Json(serde_json::json!({ "error": err.to_string() }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
        }
    }
}
