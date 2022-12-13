use axum::http::StatusCode;
use axum::{
    extract::Json,
    response::{self},
};
use serde_json::Value;

use crate::validator::{CloudConfig, Format, Validator};

pub async fn validate(Json(payload): Json<CloudConfig>) -> (StatusCode, Json<Value>) {
    // TODO: share the validator at higher level point
    let validator = Validator::new();

    let payload: Value = match payload.format() {
        Format::Yaml => serde_yaml::from_str(payload.payload()).unwrap(),
        Format::Json => serde_json::from_str(payload.payload()).unwrap(),
    };

    let resp = validator.validate(&payload);
    let resp = serde_json::to_value(resp).unwrap();
    (StatusCode::OK, response::Json(resp))
}
