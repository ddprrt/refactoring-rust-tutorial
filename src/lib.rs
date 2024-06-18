use std::sync::{Arc, RwLock};

use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use database::KeyValueStore;
use kv_store::{get_kv, grayscale, post_kv, thumbnail};
use serde::Deserialize;

mod database;
mod kv_store;
pub mod types;

/// Custom type for a shared state
pub type SharedState<T> = Arc<RwLock<AppState<T>>>;

async fn handler() -> impl IntoResponse {
    "<h1>Hello Axum</h1>"
}

#[derive(Deserialize)]
struct Name {
    name: Option<String>,
}

async fn hello_handler(Query(name): Query<Name>) -> impl IntoResponse {
    match name.name {
        Some(name) => format!("<h1>Hello {}</h1>", name),
        None => "<h1>Hello Unknown Visitor</h1>".to_string(),
    }
}
#[derive(Default)]
pub struct AppState<T: KeyValueStore> {
    db: T,
}

pub fn router<T: KeyValueStore + Send + Sync + 'static>(
    state: &SharedState<T>,
) -> Router<SharedState<T>> {
    Router::with_state(Arc::clone(state))
        .route("/", get(handler))
        .route("/hello", get(hello_handler))
        .route("/kv/:key", get(get_kv).post(post_kv))
        .route("/kv/:key/grayscale", get(grayscale))
        .route("/kv/:key/thumbnail", get(thumbnail))
}
