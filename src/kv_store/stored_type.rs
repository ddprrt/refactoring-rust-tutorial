use axum::{body::Bytes, response::IntoResponse};
use image::DynamicImage;

use super::{image_response::ImageResponse, kv_error::KVError};

#[derive(Clone)]
pub enum StoredType {
    Image(DynamicImage),
    Other((String, Bytes)),
}

impl TryFrom<(String, Bytes)> for StoredType {
    type Error = KVError;

    fn try_from((content_type, content): (String, Bytes)) -> Result<Self, Self::Error> {
        if content_type.starts_with("image/") {
            let image = image::load_from_memory(&content)?;
            Ok(StoredType::Image(image))
        } else {
            Ok(StoredType::Other((content_type, content)))
        }
    }
}

impl IntoResponse for StoredType {
    fn into_response(self) -> axum::response::Response {
        match self {
            StoredType::Image(image) => match ImageResponse::try_from(image) {
                Ok(image) => image.into_response(),
                Err(err) => err.into_response(),
            },
            StoredType::Other((content_type, content)) => {
                ([("content-type", content_type)], content).into_response()
            }
        }
    }
}
