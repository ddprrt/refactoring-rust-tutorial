use std::sync::{Arc, RwLock};

use axum::{response::IntoResponse, routing::get, Router};
use kv_store::{blur, database::KVDatabase, get_kv, grayscale, post_kv};

pub mod kv_store;

#[derive(Default)]
pub struct AppState<T: KVDatabase> {
    db: T,
}

/// Custom type for a shared state
pub type SharedState<T> = Arc<RwLock<AppState<T>>>;

async fn handler() -> impl IntoResponse {
    "<h1>Hello Axum</h1>"
}

pub fn router<T: KVDatabase + Send + Sync + 'static>(
    state: &SharedState<T>,
) -> Router<SharedState<T>> {
    Router::with_state(Arc::clone(state))
        .route("/", get(handler))
        .route("/kv/:key", get(get_kv).post(post_kv))
        .route("/kv/:key/grayscale", get(grayscale))
        .route("/kv/:key/blur/:sigma", get(blur))
}
