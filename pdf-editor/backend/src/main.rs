use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use serde::Deserialize;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod pdf;
mod types;
#[allow(dead_code)]
mod util;

use crate::pdf::loader::{PdfDocument, PdfStore};
use crate::types::{DocumentIR, PageIR, PatchOp};

static SAMPLE_PDF: &[u8] = include_bytes!("../e2e/sample.pdf");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let store = PdfStore::default();

    let app = Router::new()
        .route("/api/open", post(open_pdf))
        .route("/api/open-sample", get(open_sample))
        .route("/api/ir/:doc_id", get(get_ir))
        .route("/api/patch/:doc_id", post(apply_patch))
        .route("/api/pdf/:doc_id", get(download_pdf))
        .with_state(store);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    info!("listening", %addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

async fn open_pdf() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "upload not yet implemented")
}

async fn open_sample(State(store): State<PdfStore>) -> impl IntoResponse {
    let doc_id = uuid::Uuid::new_v4();
    let sample_ir = DocumentIR {
        pages: vec![PageIR {
            index: 0,
            width_pt: 595.0,
            height_pt: 842.0,
            objects: vec![],
        }],
    };

    let pdf_bytes = Bytes::from_static(SAMPLE_PDF);
    let document = PdfDocument {
        original_bytes: pdf_bytes.clone(),
        current_bytes: pdf_bytes.clone(),
        ir: sample_ir.clone(),
    };
    store.insert(doc_id, document);

    Json(OpenResponse {
        doc_id,
        ir: sample_ir,
    })
}

#[derive(Deserialize)]
struct DocPath {
    doc_id: uuid::Uuid,
}

async fn get_ir(
    State(store): State<PdfStore>,
    Path(DocPath { doc_id }): Path<DocPath>,
) -> Result<Json<DocumentIR>, (StatusCode, &'static str)> {
    match store.get(&doc_id) {
        Some(doc) => Ok(Json(doc.ir)),
        None => Err((StatusCode::NOT_FOUND, "unknown document")),
    }
}

async fn apply_patch(
    State(store): State<PdfStore>,
    Path(DocPath { doc_id }): Path<DocPath>,
    Json(_ops): Json<Vec<PatchOp>>,
) -> impl IntoResponse {
    if store.get(&doc_id).is_none() {
        return (StatusCode::NOT_FOUND, "unknown document").into_response();
    }

    Json(types::PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    })
}

async fn download_pdf(
    State(store): State<PdfStore>,
    Path(DocPath { doc_id }): Path<DocPath>,
) -> impl IntoResponse {
    match store.get(&doc_id) {
        Some(doc) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/pdf"),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=sample.pdf"),
            );
            (headers, doc.current_bytes).into_response()
        }
        None => (StatusCode::NOT_FOUND, "unknown document").into_response(),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenResponse {
    doc_id: uuid::Uuid,
    ir: DocumentIR,
}
