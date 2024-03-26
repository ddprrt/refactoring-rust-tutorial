use std::io::Cursor;

use axum::{
    extract::{Path, State},
    headers::ContentType,
    response::IntoResponse,
    TypedHeader,
};
use hyper::{body::Bytes, StatusCode};
use image::ImageOutputFormat;

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
        .unwrap()
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
) -> Result<impl IntoResponse, impl IntoResponse> {
    let image = match state.read().unwrap().db.get(&key) {
        Some((content_type, data)) => {
            if content_type == "image/png" {
                image::load_from_memory(&data).unwrap()
            } else {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Not possible to grayscale this type of image",
                )
                    .into_response());
            }
        }
        None => return Err((StatusCode::NOT_FOUND, "Key not found").into_response()),
    };

    let mut vec: Vec<u8> = Vec::new();

    let mut cursor = Cursor::new(&mut vec);
    image
        .grayscale()
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .unwrap();
    let bytes: Bytes = vec.into();

    Ok(([("content-type", "image/png")], bytes).into_response())
}
