use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use microservice_rust_workshop::{router, SharedState};
use tower::Service; // for `call`

#[tokio::test]
async fn basic_db_test() {
    let state = SharedState::default();
    let mut app = router(&state);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("POST")
                .header("content-type", "text/plain")
                .body("Hello World".into())
                .unwrap(),
        )
        .await
        .unwrap();

    //assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let bytes = body.to_vec();
    assert_eq!(String::from_utf8(bytes).unwrap(), "OK".to_string());

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let bytes = body.to_vec();
    assert_eq!(String::from_utf8(bytes).unwrap(), "Hello World".to_string());
}

#[tokio::test]
async fn image_request() {
    let state = SharedState::default();
    let mut app = router(&state);
    let bytes = include_bytes!("../crab-small.png");

    let response = app
        .call(
            Request::builder()
                .uri("/kv/crab")
                .method("POST")
                .header("content-type", "image/png")
                .body(bytes[..].into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/crab")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn no_entry() {
    let state = SharedState::default();
    let mut app = router(&state);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/crab")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn grayscale_request() {
    let state = SharedState::default();
    let mut app = router(&state);
    let bytes = include_bytes!("../crab-small.png");
    let grayscale = include_bytes!("../crab-small-grayscale.png");

    let response = app
        .call(
            Request::builder()
                .uri("/kv/crab")
                .method("POST")
                .header("content-type", "image/png")
                .body(bytes[..].into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/crab/grayscale")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], &grayscale[..]);
}

#[tokio::test]
async fn grayscale_faulty_request() {
    let state = SharedState::default();
    let mut app = router(&state);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("POST")
                .header("content-type", "text/plain")
                .body("Hello World".into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test/grayscale")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
