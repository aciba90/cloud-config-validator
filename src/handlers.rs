use axum::http::StatusCode;
use axum::{
    extract::Json,
    response::{self},
};
use serde_json::Value;

use crate::validator::{CloudConfig, Validator};

pub async fn validate(Json(payload): Json<CloudConfig>) -> (StatusCode, Json<Value>) {
    // TODO: share the validator at higher level point
    let validator = Validator::new();
    let mut status_code = StatusCode::OK;
    let resp = match validator.validate_yaml(&payload.payload()) {
        Ok(v) => Ok(v),
        Err(e) => {
            status_code = StatusCode::BAD_REQUEST;
            Err(e)
        }
    };
    let resp = serde_json::to_value(resp).expect("Validation response must be serializable");
    (status_code, response::Json(resp))
}