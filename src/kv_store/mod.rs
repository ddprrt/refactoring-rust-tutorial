use axum::{
    extract::{Path, State},
    headers::ContentType,
    TypedHeader,
};
use database::KVDatabase;
use hyper::body::Bytes;
use image_response::ImageResponse;
use kv_error::KVError;
use stored_type::StoredType;

use crate::SharedState;

pub mod database;
mod image_response;
mod kv_error;
pub mod stored_type;

pub async fn post_kv<T: KVDatabase>(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState<T>>,
    data: Bytes,
) -> Result<String, KVError> {
    state
        .write()?
        .db
        .insert(key, StoredType::try_from((content_type.to_string(), data))?)?;
    Ok("OK".to_string())
}

pub async fn get_kv<T: KVDatabase>(
    Path(key): Path<String>,
    State(state): State<SharedState<T>>,
) -> Result<StoredType, KVError> {
    state.read()?.db.read(key)
}

pub async fn blur<T: KVDatabase>(
    Path((key, sigma)): Path<(String, f32)>,
    State(state): State<SharedState<T>>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.read(key)? {
        StoredType::Image(image) => Ok(image.blur(sigma).try_into()?),
        _ => Err(KVError::forbidden()),
    }
}

pub async fn grayscale<T: KVDatabase>(
    Path(key): Path<String>,
    State(state): State<SharedState<T>>,
) -> Result<ImageResponse, KVError> {
    match state.read()?.db.read(key)? {
        StoredType::Image(image) => Ok(image.grayscale().try_into()?),
        _ => Err(KVError::forbidden()),
    }
}
