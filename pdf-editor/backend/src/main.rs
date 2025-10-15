use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    body::Full,
    extract::{Multipart, Path, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use pdf::{extract, loader::DocumentStore, patch};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use types::{DocumentIr, PatchOp};
use uuid::Uuid;

mod pdf;
mod types;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        store: Arc::new(DocumentStore::new()),
    };

    let app = Router::new()
        .route("/api/open", post(open_document))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    println!("Backend listening on {addr}");
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    store: Arc<DocumentStore>,
}

async fn open_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OpenResponse>, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(ApiError::bad_request)?
    {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(ApiError::bad_request)?;
            let id = state.store.insert(data.to_vec());
            return Ok(Json(OpenResponse { doc_id: id }));
        }
    }
    Err(ApiError::bad_request("file field missing"))
}

async fn get_ir(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<DocumentIr>, ApiError> {
    let document = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;
    let doc = document.read();
    let ir = extract::extract_ir(&doc.current).map_err(ApiError::internal)?;
    Ok(Json(ir))
}

async fn apply_patch(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
    Json(ops): Json<Vec<PatchOp>>,
) -> Result<Json<patch::PatchResponse>, ApiError> {
    let document = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;
    let mut doc = document.write();
    let result = patch::apply_patches(&mut doc.current, &ops).map_err(ApiError::internal)?;
    Ok(Json(result))
}

async fn download_pdf(
    State(state): State<AppState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let document = state
        .store
        .get(&doc_id)
        .ok_or_else(|| ApiError::not_found("document not found"))?;
    let doc = document.read();
    let mut response = Response::new(Full::from(doc.current.clone()));
    response
        .headers_mut()
        .insert(CONTENT_TYPE, "application/pdf".parse().unwrap());
    Ok(response)
}

#[derive(serde::Serialize)]
struct OpenResponse {
    #[serde(rename = "docId")]
    doc_id: Uuid,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request<E: std::fmt::Display>(error: E) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: error.to_string(),
        }
    }

    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    fn internal(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "ok": false,
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

/// Provide a minimal health endpoint for future probes.
async fn _health() -> Json<HashMap<&'static str, &'static str>> {
    Json(HashMap::from([( "status", "ok" )]))
}

