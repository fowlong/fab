use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use pdf_editor_backend::{
    build_router,
    types::{PatchOperation, PatchTarget, StylePayload},
    AppState,
};
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn open_fetch_ir_and_patch_flow() {
    let state = AppState::default();
    let app: Router<_> = build_router(state);

    let boundary = "X-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"upload.pdf\"\r\nContent-Type: application/pdf\r\n\r\nPDF\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/api/open")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    let doc_id = payload["docId"].as_str().expect("docId missing");

    let ir_request = Request::builder()
        .method("GET")
        .uri(format!("/api/ir/{doc_id}"))
        .body(Body::empty())
        .unwrap();

    let ir_response = app.clone().oneshot(ir_request).await.unwrap();
    assert_eq!(ir_response.status(), StatusCode::OK);

    let bytes = to_bytes(ir_response.into_body(), usize::MAX).await.unwrap();
    let ir: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(ir["pages"].is_array());

    let ops = vec![PatchOperation::SetStyle {
        target: PatchTarget {
            page: 0,
            id: "t:42".into(),
        },
        style: StylePayload {
            fill_color: Some([1.0, 0.0, 0.0]),
            ..Default::default()
        },
    }];

    let patch_request = Request::builder()
        .method("POST")
        .uri(format!("/api/patch/{doc_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&ops).unwrap()))
        .unwrap();

    let patch_response = app.oneshot(patch_request).await.unwrap();
    assert_eq!(patch_response.status(), StatusCode::OK);

    let bytes = to_bytes(patch_response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], true);
}
