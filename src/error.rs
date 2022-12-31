use axum::response::{self, IntoResponse};
use hyper::StatusCode;
use serde_json::json;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid yaml: {}", .0)]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("invalid json: {}", .0)]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid JsonSchema: {}", .0)]
    InvalidSchema(String),

    #[error("request error: {}", .0)]
    RequestError(#[from] reqwest::Error),

    #[error("local JsonSchema ref not found: {}", .r#ref)]
    LocalSchemaRefNotFound { r#ref: String },

    #[error("local JsonSchema with invalid name: {}", .r#ref)]
    LocalSchemaRefInvalidName { r#ref: String },
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let (status, error) = match err {
            Error::InvalidYaml(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::InvalidJson(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::InvalidSchema(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Error::RequestError(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Error::LocalSchemaRefNotFound { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            Error::LocalSchemaRefInvalidName { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        };
        Self { status, error }
    }
}

pub struct ApiError {
    status: StatusCode,
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = response::Json(json!({"errors": [self.error]}));
        (self.status, body).into_response()
    }
}
