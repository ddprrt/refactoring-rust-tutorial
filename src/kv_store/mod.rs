use axum::{
    extract::{Path, State},
    headers::ContentType,
    response::IntoResponse,
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
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(elem) => Ok(elem.into_response()),
        None => Err(KVError::not_found()),
    }
}

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => Ok(ImageResponse::try_from(image.grayscale())?),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}

pub async fn _thumbnail(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => Ok(ImageResponse::try_from(image.resize(
            100,
            100,
            image::imageops::FilterType::Nearest,
        ))?),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}
