use axum::response::{self, IntoResponse};
use ccv_core::error::Error;
use hyper::StatusCode;
use serde_json::json;

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
