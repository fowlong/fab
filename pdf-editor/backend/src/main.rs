mod pdf;
mod types;
mod util;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use hashbrown::HashMap;
use parking_lot::RwLock;
use pdf::loader::StoredDocument;
use types::{IrDocument, PatchOp, PatchResponse};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    docs: Arc<RwLock<HashMap<Uuid, StoredDocument>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        docs: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(fetch_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    println!("PDF editor backend listening on {addr}");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<types::OpenResponse>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            filename = field.file_name().map(|f| f.to_string());
            file_bytes = Some(field.bytes().await?.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or_else(|| ApiError::BadRequest("missing file field".into()))?;

    let document = StoredDocument::new(filename, bytes);
    let doc_id = document.id();

    state.docs.write().insert(doc_id, document);

    Ok(Json(types::OpenResponse { doc_id }))
}

async fn fetch_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<IrDocument>, ApiError> {
    let guard = state.docs.read();
    let doc = guard
        .get(&doc_id)
        .ok_or(ApiError::NotFound(doc_id))?
        .clone();

    let ir = pdf::extract::extract_ir(&doc)?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<PatchResponse>, ApiError> {
    let mut guard = state.docs.write();
    let doc = guard.get_mut(&doc_id).ok_or(ApiError::NotFound(doc_id))?;

    let outcome = pdf::patch::apply_patch(doc, &ops)?;

    let encoded = format!(
        "data:application/pdf;base64,{}",
        base64::encode(outcome.updated_pdf)
    );

    Ok(Json(PatchResponse {
        ok: true,
        updated_pdf: Some(encoded),
        remap: outcome.remap,
        message: Some("Patch application is stubbed in this prototype".into()),
    }))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let guard = state.docs.read();
    let doc = guard.get(&doc_id).ok_or(ApiError::NotFound(doc_id))?;

    let bytes = doc.latest_bytes().to_vec();

    let mut response = Response::new(bytes.into());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("document {0} not found")]
    NotFound(Uuid),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Multipart(_) => StatusCode::BAD_REQUEST,
            ApiError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(serde_json::json!({
            "ok": false,
            "error": self.to_string(),
        }));
        (status, body).into_response()
    }
}
