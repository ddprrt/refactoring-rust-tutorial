use axum::{
    extract::{Path, State},
    headers::ContentType,
    response::{IntoResponse, Response},
    TypedHeader,
};
use hyper::body::Bytes;

use crate::SharedState;

use self::image_response::ImageResponse;
use self::kv_error::KVError;

mod image_response;
mod kv_error;
mod stored_type;

pub use stored_type::StoredType;

pub async fn post_kv(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState>,
    data: Bytes,
) -> Result<String, KVError> {
    let stored = StoredType::new(content_type, data)?;
    state.write()?.db.insert(key, stored);
    Ok("OK".to_string())
}

pub async fn get_kv(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<StoredType, KVError> {
    match state.read()?.db.get(&key) {
        Some(elem) => Ok(elem.clone()),
        None => Err(KVError::not_found()),
    }
}

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => image.grayscale().try_into(),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}

pub async fn thumbnail(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => image.thumbnail(100, 100).try_into(),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}
