use std::sync::PoisonError;

use axum::response::IntoResponse;
use hyper::StatusCode;
use image::ImageError;

#[derive(Debug)]
pub struct KVError(StatusCode, String);

impl KVError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self(status, message.into())
    }

    pub(crate) fn not_found() -> KVError {
        KVError::new(StatusCode::NOT_FOUND, "Key not found")
    }

    pub(crate) fn forbidden() -> KVError {
        KVError::new(
            StatusCode::FORBIDDEN,
            "Not possible to grayscale this type of image",
        )
    }
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0.as_str(), self.1)
    }
}

impl std::error::Error for KVError {}

impl<T> From<PoisonError<T>> for KVError {
    fn from(_value: PoisonError<T>) -> Self {
        KVError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error writing to DB")
    }
}

impl From<ImageError> for KVError {
    fn from(_value: ImageError) -> Self {
        KVError::new(StatusCode::BAD_REQUEST, "Error processing image")
    }
}

impl IntoResponse for KVError {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}
