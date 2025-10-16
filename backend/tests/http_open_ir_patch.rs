use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use pdf_editor_backend::{
    app::{self, AppState, OpenResponse},
    types::{DocumentIR, PageObject, PatchOperation, PatchTarget},
    util::matrix::Matrix2D,
};
use tower::util::ServiceExt;

#[tokio::test]
async fn open_patch_flow_updates_document_ir() {
    let state = AppState::default();
    let app = app::app_router(state);

    let boundary = "XBOUNDARY";
    let open_request = Request::builder()
        .method("POST")
        .uri("/api/open")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(format!("--{boundary}--\r\n")))
        .unwrap();

    let open_response = app
        .clone()
        .oneshot(open_request)
        .await
        .expect("open request should succeed");
    assert_eq!(open_response.status(), StatusCode::OK);
    let body = to_bytes(open_response.into_body(), usize::MAX)
        .await
        .expect("open response body");
    let open: OpenResponse = serde_json::from_slice(&body).expect("parse open response");

    let doc_id = open.doc_id;
    assert!(doc_id.starts_with("doc-"));

    let delta = [1.0, 0.0, 0.0, 1.0, 8.0, -4.0];
    let patch_ops = vec![PatchOperation::Transform {
        target: PatchTarget {
            page: 0,
            id: "t:42".into(),
        },
        delta_matrix_pt: delta,
        kind: "text".into(),
    }];

    let patch_request = Request::builder()
        .method("POST")
        .uri(format!("/api/patch/{doc_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&patch_ops).unwrap()))
        .unwrap();

    let patch_response = app
        .clone()
        .oneshot(patch_request)
        .await
        .expect("patch request should succeed");
    assert_eq!(patch_response.status(), StatusCode::OK);

    let ir_request = Request::builder()
        .method("GET")
        .uri(format!("/api/ir/{doc_id}"))
        .body(Body::empty())
        .unwrap();

    let ir_response = app
        .oneshot(ir_request)
        .await
        .expect("ir request should succeed");
    assert_eq!(ir_response.status(), StatusCode::OK);

    let body = to_bytes(ir_response.into_body(), usize::MAX)
        .await
        .expect("ir response body");
    let ir: DocumentIR = serde_json::from_slice(&body).expect("parse ir response");

    let expected_tm = if let PageObject::Text(text) = &DocumentIR::sample().pages[0].objects[0] {
        Matrix2D::from_array(delta)
            .concat(Matrix2D::from_array(text.tm))
            .to_array()
    } else {
        panic!("sample IR missing text object");
    };

    let text_object = ir.pages[0]
        .objects
        .iter()
        .find_map(|object| match object {
            PageObject::Text(text) => Some(text),
            _ => None,
        })
        .expect("transformed text object present");

    assert_eq!(text_object.tm, expected_tm);
}
