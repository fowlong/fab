mod pdf;
mod types;
mod util;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use parking_lot::RwLock;
use pdf::{extract::extract_placeholder_ir, loader::load_pdf_bytes, patch::apply_patches};
use serde::Serialize;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use types::{DocumentIR, PatchOperation, PatchResponse};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<DocumentStore>>,
}

struct DocumentStore {
    doc_id: String,
    ir: DocumentIR,
    pdf_bytes: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let pdf_bytes = load_pdf_bytes("../e2e/sample.pdf")?;
    let ir = extract_placeholder_ir()?;

    let state = AppState {
        store: Arc::new(RwLock::new(DocumentStore {
            doc_id: "sample".to_string(),
            ir,
            pdf_bytes,
        })),
    };

    let app = Router::new()
        .route("/api/open", post(open_doc))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse().unwrap();
    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
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
    info!("shutdown signal received");
}

async fn open_doc(state: axum::extract::State<AppState>) -> impl IntoResponse {
    let doc_id = state.store.read().doc_id.clone();
    Json(json!({"docId": doc_id}))
}

async fn fetch_ir(
    state: axum::extract::State<AppState>,
    Path(doc_id): Path<String>,
) -> impl IntoResponse {
    match with_doc(state, doc_id, |doc| Json(&doc.ir)).await {
        Ok(response) => response.into_response(),
        Err(err) => err.into_response(),
    }
}

async fn apply_patch(
    state: axum::extract::State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> impl IntoResponse {
    let result = with_doc_mut(state, doc_id, |doc| {
        if let Err(err) = apply_patches(&mut doc.ir, &ops) {
            error!(?err, "failed to apply patch");
            return Err((StatusCode::BAD_REQUEST, err.to_string()));
        }
        Ok(Json(PatchResponse {
            ok: true,
            updated_pdf: Some(BASE64_STANDARD.encode(&doc.pdf_bytes)),
            remap: None,
        }))
    })
    .await;

    match result {
        Ok(response) => response.into_response(),
        Err(err) => err.into_response(),
    }
}

async fn download_pdf(
    state: axum::extract::State<AppState>,
    Path(doc_id): Path<String>,
) -> impl IntoResponse {
    match with_doc(state, doc_id, |doc| {
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "application/pdf")
            .body(axum::body::Body::from(doc.pdf_bytes.clone()))
            .unwrap()
    })
    .await
    {
        Ok(response) => response.into_response(),
        Err(err) => err.into_response(),
    }
}

async fn with_doc<T, F>(
    state: axum::extract::State<AppState>,
    doc_id: String,
    f: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(&DocumentStore) -> T,
    T: IntoResponse,
{
    let guard = state.store.read();
    if guard.doc_id != doc_id {
        return Err(ResponseError::not_found());
    }
    Ok(f(&guard))
}

async fn with_doc_mut<T, F>(
    state: axum::extract::State<AppState>,
    doc_id: String,
    f: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(&mut DocumentStore) -> Result<T, (StatusCode, String)>,
{
    let mut guard = state.store.write();
    if guard.doc_id != doc_id {
        return Err(ResponseError::not_found());
    }
    f(&mut guard).map_err(|(status, message)| ResponseError { status, message })
}

#[derive(Debug)]
struct ResponseError {
    status: StatusCode,
    message: String,
}

impl ResponseError {
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "document not found".to_string(),
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

use axum::response::Response;
use serde_json::json;
