use std::net::SocketAddr;

use axum::{extract::State, routing::get, routing::post, Json, Router};
use tokio::signal;

use crate::pdf::loader::DocumentStore;
use crate::types::{OpenResponse, PatchRequest};

mod types;
mod util;
mod pdf;

#[derive(Clone)]
struct AppState {
  store: DocumentStore,
}

impl Default for AppState {
  fn default() -> Self {
    Self {
      store: DocumentStore::default(),
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let state = AppState::default();
  let app = Router::new()
    .route("/api/open", post(open))
    .route("/api/ir/:doc_id", get(ir))
    .route("/api/patch/:doc_id", post(patch))
    .route("/api/pdf/:doc_id", get(download))
    .with_state(state);

  let addr: SocketAddr = "0.0.0.0:8787".parse()?;
  println!("listening on {}", addr);
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

async fn open(State(state): State<AppState>) -> Json<OpenResponse> {
  state.store.insert("demo".into(), Vec::new());
  Json(OpenResponse {
    doc_id: "demo".into(),
  })
}

async fn ir() -> Json<serde_json::Value> {
  Json(serde_json::json!({ "pages": [] }))
}

async fn patch(Json(_req): Json<PatchRequest>) -> Json<serde_json::Value> {
  Json(serde_json::json!({ "ok": true }))
}

async fn download() -> axum::response::Response {
  axum::response::Response::new(axum::body::Body::empty())
}
