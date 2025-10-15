use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pdf;
mod types;
mod util;

use pdf::{loader::DocumentStore, patch::PatchResult};
use types::{DocumentIR, PatchOperation};

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: Arc::new(DocumentStore::default()),
    };

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 8787).into();
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn open_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, AppError> {
    let mut pdf_bytes = None;
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            let data = field.bytes().await?.to_vec();
            pdf_bytes = Some(data);
            break;
        }
    }

    let bytes = pdf_bytes.ok_or_else(|| AppError::bad_request("missing file field"))?;
    let doc_id = state.store.insert(bytes).await?;
    let ir = state.store.ir(&doc_id).await?;

    Ok(Json(OpenResponse { doc_id, ir }))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentIR>, AppError> {
    let ir = state.store.ir(&doc_id).await?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(patch): Json<Vec<PatchOperation>>,
) -> Result<Json<PatchResponse>, AppError> {
    let result = state.store.apply_patch(&doc_id, patch).await?;
    Ok(Json(PatchResponse::from(result)))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Response, AppError> {
    let bytes = state.store.latest_pdf(&doc_id).await?;
    let body = Body::from(bytes);
    let mut response = Response::new(body);
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/pdf"),
    );
    Ok(response)
}

#[derive(Debug)]
struct AppError {
    code: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut map = HashMap::new();
        map.insert("error", self.message);
        let body = Json(map);
        (self.code, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(?err, "internal error");
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal error".into(),
        }
    }
}

impl From<axum::Error> for AppError {
    fn from(err: axum::Error) -> Self {
        tracing::error!(?err, "axum error");
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal error".into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: String,
    ir: DocumentIR,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchResponse {
    ok: bool,
    updated_pdf: Option<String>,
    remap: HashMap<String, serde_json::Value>,
}

impl From<PatchResult> for PatchResponse {
    fn from(result: PatchResult) -> Self {
        Self {
            ok: result.ok,
            updated_pdf: result.updated_pdf,
            remap: result.remap,
        }
    }
}
