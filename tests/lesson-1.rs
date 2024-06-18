use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use microservice_rust_workshop::{router, SharedState};
use tower::Service; // for `call`

#[tokio::test]
async fn hello_world() {
    let state = SharedState::default();
    let mut app = router(&state);

    let response = app
        .call(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"<h1>Hello Axum</h1>");
}
