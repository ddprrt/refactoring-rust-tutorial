use std::io::Cursor;

use axum::{
    extract::{Path, State},
    headers::ContentType,
    response::IntoResponse,
    TypedHeader,
};
use hyper::{body::Bytes, StatusCode};
use image::{DynamicImage, ImageOutputFormat};
use kv_error::KVError;

use crate::SharedState;

mod kv_error;

pub async fn post_kv(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState>,
    data: Bytes,
) -> Result<String, ()> {
    state
        .write()
        .expect("What, an error here?")
        .db
        .insert(key, (content_type.to_string(), data));
    Ok("OK".to_string())
}

pub async fn get_kv(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match state.read().unwrap().db.get(&key) {
        Some((content_type, data)) => Ok(([("content-type", content_type.clone())], data.clone())),
        None => Err((StatusCode::NOT_FOUND, "Key not found").into_response()),
    }
}

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    let image = get_image(state, key)?;

    let mut vec: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut vec);
    image
        .grayscale()
        .write_to(&mut cursor, ImageOutputFormat::Png)?;
    let bytes: Bytes = vec.into();
    Ok(([("content-type", "image/png")], bytes).into_response())
}

fn get_image(state: SharedState, key: String) -> Result<DynamicImage, KVError> {
    let db = &state.read()?.db;
    let Some((content_type, data)) = db.get(&key) else {
        return Err(KVError::not_found())
    };
    let image = if content_type == "image/png" {
        image::load_from_memory(&data)?
    } else {
        return Err(KVError::forbidden());
    };
    Ok(image)
}
