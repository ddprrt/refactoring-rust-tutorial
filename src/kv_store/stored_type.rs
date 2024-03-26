use axum::{headers::ContentType, response::IntoResponse};
use hyper::body::Bytes;
use image::DynamicImage;

use super::{image_response::ImageResponse, kv_error::KVError};

pub enum StoredType {
    Image(DynamicImage),
    Other(ContentType, Bytes),
}

impl StoredType {
    pub fn new(content_type: ContentType, bytes: Bytes) -> Result<Self, KVError> {
        if content_type.to_string().starts_with("image") {
            let image = image::load_from_memory(&bytes)?;
            Ok(StoredType::Image(image))
        } else {
            Ok(StoredType::Other(content_type, bytes))
        }
    }
}

impl IntoResponse for &StoredType {
    fn into_response(self) -> axum::response::Response {
        match self {
            StoredType::Image(image) => match ImageResponse::try_from(image) {
                Ok(response) => response.into_response(),
                Err(image_error) => KVError::from(image_error).into_response(),
            },
            StoredType::Other(content_type, bytes) => {
                ([("content-type", content_type.to_string())], bytes.clone()).into_response()
            }
        }
    }
}
