use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Path, routing::get, routing::post, Json, Router};
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use crate::pdf::loader::DocumentStore;
use crate::types::{DocumentIR, OpenResponse, PatchOperation, PatchResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
      std::env::var("RUST_LOG").unwrap_or_else(|_| "pdf_editor_backend=debug,tower_http=debug".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();

  let store = Arc::new(RwLock::new(DocumentStore::default()));

  let app = Router::new()
    .route("/api/open", post(open_document))
    .route("/api/ir/:doc_id", get(get_ir))
    .route("/api/patch/:doc_id", post(apply_patch))
    .route("/api/pdf/:doc_id", get(download_pdf))
    .with_state(store);

  let addr: SocketAddr = "0.0.0.0:8787".parse()?;
  tracing::info!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;

  Ok(())
}

async fn open_document(
  axum::extract::State(store): axum::extract::State<Arc<RwLock<DocumentStore>>>,
  mut multipart: axum::extract::Multipart,
) -> Result<Json<OpenResponse>, axum::http::StatusCode> {
  let mut file_bytes = None;

  while let Some(field) = multipart.next_field().await.unwrap_or(None) {
    if field.name() == Some("file") {
      file_bytes = Some(field.bytes().await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?);
      break;
    }
  }

  let Some(bytes) = file_bytes else {
    return Err(axum::http::StatusCode::BAD_REQUEST);
  };

  let mut store = store.write().await;
  let doc_id = store.insert(bytes.to_vec());

  Ok(Json(OpenResponse { doc_id }))
}

async fn get_ir(
  axum::extract::State(store): axum::extract::State<Arc<RwLock<DocumentStore>>>,
  Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, axum::http::StatusCode> {
  let store = store.read().await;
  let doc = store
    .get(&doc_id)
    .ok_or(axum::http::StatusCode::NOT_FOUND)?
    .clone();

  let ir = pdf::extract::extract_ir(&doc_id, &doc).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(ir))
}

async fn apply_patch(
  axum::extract::State(store): axum::extract::State<Arc<RwLock<DocumentStore>>>,
  Path(doc_id): Path<String>,
  Json(ops): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, axum::http::StatusCode> {
  let mut store = store.write().await;
  let doc = store
    .get_mut(&doc_id)
    .ok_or(axum::http::StatusCode::NOT_FOUND)?;

  let response = pdf::patch::apply_patches(&doc_id, doc, &ops)
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(response))
}

async fn download_pdf(
  axum::extract::State(store): axum::extract::State<Arc<RwLock<DocumentStore>>>,
  Path(doc_id): Path<String>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
  let store = store.read().await;
  let doc = store
    .get(&doc_id)
    .ok_or(axum::http::StatusCode::NOT_FOUND)?;

  let mut response = axum::http::Response::new(axum::body::Body::from(doc.bytes.clone()));
  response.headers_mut().insert(
    axum::http::header::CONTENT_TYPE,
    axum::http::HeaderValue::from_static("application/pdf"),
  );
  Ok(response)
}
