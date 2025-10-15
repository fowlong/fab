use std::net::SocketAddr;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tokio::signal;
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::extract::extract_ir;
use pdf::loader::DocumentStore;
use pdf::patch::apply_patch;
use pdf::write::incremental_save;
use types::{IrDocument, PatchOperation, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    documents: DocumentStore,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        documents: DocumentStore::new(),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch_route))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    println!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
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

async fn open_document(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let doc_id = Uuid::new_v4().to_string();
    state.documents.insert(doc_id.clone(), body.to_vec());
    (StatusCode::CREATED, Json(json!({ "docId": doc_id })))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<IrDocument>, AppError> {
    let Some(bytes) = state.documents.get(&doc_id) else {
        return Err(AppError::not_found());
    };
    let ir = extract_ir(&bytes)?;
    Ok(Json(ir))
}

async fn apply_patch_route(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, AppError> {
    let Some(mut bytes) = state.documents.get(&doc_id) else {
        return Err(AppError::not_found());
    };
    apply_patch(&mut bytes, &ops)?;
    let updated = incremental_save(&bytes)?;
    state.documents.update(&doc_id, updated.clone())?;
    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(format!(
            "data:application/pdf;base64,{}",
            base64::encode(updated)
        )),
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let Some(bytes) = state.documents.get(&doc_id) else {
        return Err(AppError::not_found());
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/pdf".parse().unwrap(),
    );
    Ok((headers, bytes))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl AppError {
    fn not_found() -> Self {
        Self(anyhow::anyhow!("document not found"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = if self.0.to_string() == "document not found" {
            StatusCode::NOT_FOUND
        } else if self.0.to_string().contains("not yet implemented") {
            StatusCode::NOT_IMPLEMENTED
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        let payload = json!({
            "error": self.0.to_string(),
        });
        (status, Json(payload)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
