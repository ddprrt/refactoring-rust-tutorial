use std::sync::PoisonError;

use axum::response::IntoResponse;
use hyper::StatusCode;
use image::ImageError;

// Type - (Standard Library) Traits - (Ecosystem) Traits

#[derive(Debug)]
pub struct KVError {
    status_code: StatusCode,
    message: String,
}

impl KVError {
    pub fn forbidden() -> Self {
        KVError {
            status_code: StatusCode::FORBIDDEN,
            message: "Operation not allowed".to_string(),
        }
    }

    pub fn not_found() -> Self {
        KVError {
            status_code: StatusCode::NOT_FOUND,
            message: "Key not found".to_string(),
        }
    }

    pub fn impossible_operation() -> Self {
        KVError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Not possible to insert value".to_string(),
        }
    }
}

impl std::error::Error for KVError {}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status_code, self.message)
    }
}

impl<T> From<PoisonError<T>> for KVError {
    fn from(_: PoisonError<T>) -> Self {
        KVError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Database not reachable".to_string(),
        }
    }
}

impl From<ImageError> for KVError {
    fn from(_: ImageError) -> Self {
        KVError {
            status_code: StatusCode::BAD_REQUEST,
            message: "Image not loadable".to_string(),
        }
    }
}

impl IntoResponse for KVError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.message).into_response()
    }
}
