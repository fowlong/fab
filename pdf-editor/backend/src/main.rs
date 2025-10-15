use std::{collections::HashMap, sync::Arc};

use axum::{extract::Multipart, extract::Path, extract::State, http::StatusCode, response::IntoResponse, routing::get, routing::post, Json, Router};
use parking_lot::RwLock;
use serde::Serialize;
use tokio::signal;
use tracing::{error, info};

mod pdf;
mod types;
mod util;

use crate::types::{DocumentIR, PatchOp, PatchResponse};

#[derive(Clone, Default)]
struct AppState {
    docs: Arc<RwLock<HashMap<String, StoredDoc>>>,
}

struct StoredDoc {
    bytes: Vec<u8>,
    ir: DocumentIR,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/api/open", post(open))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr = "0.0.0.0:8787".parse()?;
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
        use tokio::signal::unix::{signal, SignalKind};

        signal(SignalKind::terminate())
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

    info!("signal received, starting graceful shutdown");
}

async fn open(State(state): State<AppState>, mut payload: Multipart) -> impl IntoResponse {
    let mut bytes: Option<Vec<u8>> = None;
    while let Some(field_result) = payload.next_field().await {
        let field = match field_result {
            Ok(field) => field,
            Err(err) => {
                error!(?err, "failed to read upload field");
                return StatusCode::BAD_REQUEST.into_response();
            }
        };
        if let Some(name) = field.name() {
            if name == "file" {
                match field.bytes().await {
                    Ok(content) => {
                        bytes = Some(content.to_vec());
                        break;
                    }
                    Err(err) => {
                        error!(?err, "failed to read upload field");
                        return StatusCode::BAD_REQUEST.into_response();
                    }
                }
            }
        }
    }

    let bytes = match bytes {
        Some(b) => b,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let doc_id = uuid::Uuid::new_v4().to_string();

    let ir = match pdf::extract::extract_document(&bytes) {
        Ok(ir) => ir,
        Err(err) => {
            error!(?err, "failed to extract document");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    state.docs.write().insert(doc_id.clone(), StoredDoc { bytes, ir });

    #[derive(Serialize)]
    struct Response<'a> {
        doc_id: &'a str,
    }

    Json(Response { doc_id: &doc_id })
}

async fn get_ir(State(state): State<AppState>, Path(doc_id): Path<String>) -> impl IntoResponse {
    let docs = state.docs.read();
    let Some(doc) = docs.get(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    Json(&doc.ir)
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(ops): Json<Vec<PatchOp>>,
) -> impl IntoResponse {
    let mut docs = state.docs.write();
    let Some(doc) = docs.get_mut(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match pdf::patch::apply_patch(&mut doc.bytes, &mut doc.ir, &ops) {
        Ok(updated_pdf) => Json(PatchResponse {
            ok: true,
            updated_pdf: Some(format!(
                "data:application/pdf;base64,{}",
                base64::encode(&updated_pdf)
            )),
            remap: None,
        })
        .into_response(),
        Err(err) => {
            error!(?err, "failed to apply patch");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(PatchResponse::error(err)))
                .into_response()
        }
    }
}

async fn download_pdf(State(state): State<AppState>, Path(doc_id): Path<String>) -> impl IntoResponse {
    let docs = state.docs.read();
    let Some(doc) = docs.get(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/pdf"),
    );

    (headers, doc.bytes.clone())
}
