use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

use pdf::loader::DocumentStore;
use types::{DocumentIR, PatchOperation, PatchResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        store: Arc::new(DocumentStore::default()),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response(),
            AppError::Other(err) => {
                eprintln!("{err:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "internal server error" })),
                )
                    .into_response()
            }
        }
    }
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("file") {
            let filename = field.file_name().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            let record = state.store.insert(bytes.to_vec(), filename);
            return Ok(Json(serde_json::json!({ "docId": record.doc_id })));
        }
    }

    Err(AppError::BadRequest("missing file field".into()))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<DocumentIR>> {
    let ir = state
        .store
        .get_or_extract_ir(&doc_id)
        .ok_or_else(|| AppError::BadRequest("unknown document".into()))?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOperation>>,
) -> AppResult<Json<PatchResponse>> {
    let response = state
        .store
        .apply_patch(&doc_id, ops)
        .ok_or_else(|| AppError::BadRequest("unknown document".into()))?;
    Ok(Json(response))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Response> {
    let Some(bytes) = state.store.get_pdf(&doc_id) else {
        return Err(AppError::BadRequest("unknown document".into()));
    };

    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        bytes,
    )
        .into_response())
}
