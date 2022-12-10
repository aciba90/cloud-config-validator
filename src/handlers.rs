use axum::extract::Json;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CloudConfig {
    format: Format,
    payload: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Yaml,
    Json,
}

pub async fn validate(Json(payload): Json<CloudConfig>) -> String {
    let resp = format!("email: {}, format: {:?}", payload.payload, payload.format);
    dbg!(&payload);
    let payload: Value = match payload.format {
        Format::Yaml => serde_yaml::from_str(&payload.payload).unwrap(),
        Format::Json => serde_json::from_str(&payload.payload).unwrap(),
    };
    dbg!(&payload);
    resp
}
