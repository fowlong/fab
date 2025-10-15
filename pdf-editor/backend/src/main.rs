use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod types;

use crate::types::{DocumentIr, PatchRequest};

#[derive(Clone, Default)]
struct AppState {
    docs: Arc<parking_lot::RwLock<Vec<DocumentIr>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter("pdf_editor_backend=info")
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let state = AppState::default();
    let app = Router::new()
        .route("/api/open", get(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", axum::routing::post(apply_patch))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    info!(%addr, "starting server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    info!("received shutdown signal");
}

async fn open_document(State(state): State<AppState>) -> impl IntoResponse {
    let mut docs = state.docs.write();
    let doc = DocumentIr::empty();
    docs.push(doc);
    #[derive(Serialize)]
    struct OpenResponse {
        doc_id: usize;
    }
    let response = OpenResponse {
        doc_id: docs.len() - 1,
    };
    (StatusCode::OK, Json(response))
}

async fn get_ir(State(state): State<AppState>, axum::extract::Path(doc_id): axum::extract::Path<usize>) -> impl IntoResponse {
    let docs = state.docs.read();
    if let Some(doc) = docs.get(doc_id) {
        Json(doc.clone()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn apply_patch(
    State(state): State<AppState>,
    axum::extract::Path(doc_id): axum::extract::Path<usize>,
    Json(patch): Json<PatchRequest>,
) -> impl IntoResponse {
    let mut docs = state.docs.write();
    let Some(doc) = docs.get_mut(doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    doc.patches.push(patch);
    #[derive(Serialize)]
    struct PatchResponse<'a> {
        ok: bool;
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_pdf: Option<&'a str>;
    }
    let response = PatchResponse {
        ok: true,
        updated_pdf: None,
    };
    Json(response).into_response()
}
