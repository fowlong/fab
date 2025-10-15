use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use serde::Serialize;
use thiserror::Error;
use tokio::{net::TcpListener, signal};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::{loader, patch};
use types::{DocumentIr, PatchOp};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<Uuid, DocumentEntry>>>,
}

struct DocumentEntry {
    original: Vec<u8>,
    current: Vec<u8>,
    ir: DocumentIr,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Message(_) => StatusCode::BAD_REQUEST,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = self.to_string();
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Serialize)]
struct OpenResponse {
    doc_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/open", post(open_handler))
        .route("/api/ir/:doc_id", get(ir_handler))
        .route("/api/pdf/:doc_id", get(pdf_handler))
        .route("/api/patch/:doc_id", post(patch_handler))
        .with_state(state);

    let addr = "0.0.0.0:8787";
    let listener = TcpListener::bind(addr).await?;
    println!("PDF editor backend listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn open_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, AppError> {
    let mut pdf_bytes = None;
    while let Some(field) = multipart.next_field().await? {
        if let Some(name) = field.name() {
            if name == "file" {
                let data = field.bytes().await?.to_vec();
                pdf_bytes = Some(data);
                break;
            }
        }
    }
    let bytes = pdf_bytes.ok_or_else(|| AppError::Message("file part missing".into()))?;
    let loaded = loader::load_from_bytes(&bytes)?;
    let doc_id = Uuid::new_v4();

    state.store.write().insert(
        doc_id,
        DocumentEntry {
            original: bytes.clone(),
            current: bytes,
            ir: loaded.ir,
        },
    );

    Ok(Json(OpenResponse {
        doc_id: doc_id.to_string(),
    }))
}

async fn ir_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIr>, AppError> {
    let uuid = Uuid::parse_str(&doc_id).context("invalid doc id")?;
    let store = state.store.read();
    let entry = store
        .get(&uuid)
        .ok_or_else(|| AppError::Message("document not found".into()))?;
    Ok(Json(entry.ir.clone()))
}

async fn pdf_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, AppError> {
    let uuid = Uuid::parse_str(&doc_id).context("invalid doc id")?;
    let store = state.store.read();
    let entry = store
        .get(&uuid)
        .ok_or_else(|| AppError::Message("document not found".into()))?;
    let mut response = Response::new(Body::from(entry.current.clone()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

async fn patch_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<types::PatchResponse>, AppError> {
    let uuid = Uuid::parse_str(&doc_id).context("invalid doc id")?;
    let mut store = state.store.write();
    let entry = store
        .get_mut(&uuid)
        .ok_or_else(|| AppError::Message("document not found".into()))?;
    let response = patch::apply_patches(&mut entry.ir, &entry.current, &ops)?;
    Ok(Json(response))
}
