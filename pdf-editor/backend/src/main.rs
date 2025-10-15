use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

mod pdf;
mod types;
mod util;

use pdf::loader::DocumentStore;
use types::{DocumentIr, PatchOp, PatchResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let store = Arc::new(Mutex::new(DocumentStore::default()));

  let app = Router::new()
    .route("/api/open", post(open))
    .route("/api/ir/:id", get(ir))
    .route("/api/patch/:id", post(patch))
    .route("/api/pdf/:id", get(download))
    .with_state(store);

  let addr: SocketAddr = "0.0.0.0:8787".parse()?;
  info!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;
  Ok(())
}

async fn open(
  state: axum::extract::State<Arc<Mutex<DocumentStore>>>,
  mut multipart: Multipart,
) -> Result<Json<types::OpenResponse>, AppError> {
  while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
    if field.name() == Some("file") {
      let data = field.bytes().await.map_err(AppError::from)?;
      let mut guard = state.lock().await;
      let doc_id = guard.insert(data.to_vec());
      drop(guard);
      let ir = ir_impl(state.clone(), doc_id.clone()).await?;
      return Ok(Json(types::OpenResponse { doc_id, ir }));
    }
  }
  Err(AppError::BadRequest("file field missing".into()))
}

async fn ir(
  axum::extract::State(state): axum::extract::State<Arc<Mutex<DocumentStore>>>,
  axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<DocumentIr>, AppError> {
  ir_impl(state, id).await.map(Json)
}

async fn ir_impl(
  state: Arc<Mutex<DocumentStore>>,
  id: String,
) -> Result<DocumentIr, AppError> {
  let guard = state.lock().await;
  let data = guard
    .get(&id)
    .ok_or_else(|| AppError::NotFound("unknown document".into()))?;
  pdf::extract::extract_ir(&data).map_err(AppError::from)
}

async fn patch(
  axum::extract::State(state): axum::extract::State<Arc<Mutex<DocumentStore>>>,
  axum::extract::Path(id): axum::extract::Path<String>,
  Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<PatchResponse>, AppError> {
  let mut guard = state.lock().await;
  let data = guard
    .get_mut(&id)
    .ok_or_else(|| AppError::NotFound("unknown document".into()))?;
  let response = pdf::patch::apply_patches(data, &ops).map_err(AppError::from)?;
  Ok(Json(response))
}

async fn download(
  axum::extract::State(state): axum::extract::State<Arc<Mutex<DocumentStore>>>,
  axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
  let guard = state.lock().await;
  let data = guard
    .get(&id)
    .ok_or_else(|| AppError::NotFound("unknown document".into()))?;
  Ok((
    StatusCode::OK,
    [("Content-Type", "application/pdf"), ("Content-Disposition", "attachment; filename=edited.pdf")],
    data.clone(),
  ))
}

#[derive(Debug)]
pub enum AppError {
  BadRequest(String),
  NotFound(String),
  Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
  fn from(err: anyhow::Error) -> Self {
    AppError::Internal(err)
  }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
  fn from(err: axum::extract::multipart::MultipartError) -> Self {
    AppError::Internal(err.into())
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let (status, message) = match &self {
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
      AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
      AppError::Internal(err) => {
        error!("internal error: {err:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
      }
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
  }
}
