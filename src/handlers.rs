use axum::{extract::Json, response::{IntoResponse, self}};
use serde_json::Value;
use axum::http::StatusCode;

use crate::validator::{CloudConfig, Format, Validator, Validation};
use serde::Serialize;

pub async fn validate(Json(payload): Json<CloudConfig>) -> (StatusCode, Json<Value>)
//jsonschema::output::BasicOutput<'static>
{
    let validator = Validator::new();

    let resp = format!(
        "email: {}, format: {:?}",
        payload.payload(),
        payload.format()
    );
    dbg!(&payload);
    let payload: Value = match payload.format() {
        Format::Yaml => serde_yaml::from_str(&payload.payload()).unwrap(),
        Format::Json => serde_json::from_str(&payload.payload()).unwrap(),
    };
    dbg!(&payload);
    let resp = validator.validate(&payload);
    let resp = serde_json::to_value(resp).unwrap();
    (StatusCode::OK, response::Json(resp) )
    // resp
}
