use std::net::SocketAddr;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod types;
mod pdf;
mod util;

use types::{DocumentIR, OpenResponse, PatchOp, PatchResponse};

#[derive(Default)]
struct AppState {
    #[allow(dead_code)]
    documents: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/api/open", post(open))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    info!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn open(State(_state): State<Arc<AppState>>) -> Json<OpenResponse> {
    Json(OpenResponse {
        doc_id: "demo".into(),
    })
}

async fn get_ir(State(_state): State<Arc<AppState>>) -> Json<DocumentIR> {
    Json(DocumentIR { pages: vec![] })
}

async fn patch(State(_state): State<Arc<AppState>>, Json(_ops): Json<Vec<PatchOp>>) -> Json<PatchResponse> {
    Json(PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    })
}

async fn download_pdf(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "PDF streaming not yet implemented")
}
