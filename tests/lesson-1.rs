use std::collections::HashMap;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use microservice_rust_workshop::{kv_store::stored_type::StoredType, router, SharedState};
use tower::Service; // for `call`

type TestState = SharedState<HashMap<String, StoredType>>;

#[tokio::test]
async fn hello_world() {
    let state = TestState::default();
    let mut app = router(&state);

    let response = app
        .call(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"<h1>Hello Axum</h1>");
}
