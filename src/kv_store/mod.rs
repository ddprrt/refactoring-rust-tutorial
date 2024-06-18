use axum::{
    extract::{Path, State},
    headers::ContentType,
    TypedHeader,
};
use hyper::body::Bytes;

use crate::{database::KeyValueStore, types::StoredType, SharedState};

use self::image_response::ImageResponse;
use self::kv_error::KVError;

pub mod image_response;
pub mod kv_error;

pub async fn post_kv<T: KeyValueStore>(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState<T>>,
    data: Bytes,
) -> Result<String, KVError> {
    let stored = StoredType::new(content_type, data)?;
    state.write()?.db.set_item(key, stored);
    Ok("OK".to_string())
}

pub async fn get_kv<T: KeyValueStore>(
    Path(key): Path<String>,
    State(state): State<SharedState<T>>,
) -> Result<StoredType, KVError> {
    match state.read()?.db.get_item(key) {
        Some(elem) => Ok(elem.clone()),
        None => Err(KVError::not_found()),
    }
}

pub async fn grayscale<T: KeyValueStore>(
    Path(key): Path<String>,
    State(state): State<SharedState<T>>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.get_item(key) {
        Some(StoredType::Image(image)) => image.grayscale().try_into(),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}

pub async fn thumbnail<T: KeyValueStore>(
    Path(key): Path<String>,
    State(state): State<SharedState<T>>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.get_item(key) {
        Some(StoredType::Image(image)) => image.thumbnail(100, 100).try_into(),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}
