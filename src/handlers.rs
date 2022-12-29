use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{extract::Json, response};
use serde_json::{json, Value};
use std::sync::RwLock;

use crate::state::AppState;
use crate::validator::CloudConfig;

#[tracing::instrument(level = "info", skip(state, payload))] // do not leak the payload
pub async fn validate(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<CloudConfig>,
) -> (StatusCode, Json<Value>) {
    let mut status_code = StatusCode::OK;

    // Note: `validate_yaml` over an unbound yaml could block the async runtime, causing a delay
    // in reponse time.
    // More info: https://ryhl.io/blog/async-what-is-blocking/

    // Option 1: Blocking
    // let resp = state.validator.validate_yaml(payload.payload());

    // Option 2: Spawn a tokio::task per request. The worker could spawn ~500 threads, which could
    // also end up blocking the async runtime.
    // let resp = tokio::task::spawn_blocking(move || state.validator.validate_yaml(payload.payload()))
    //    .await
    //    .expect("Panic in tokio::task::spawn_blocking");

    // Option 3: Spawn in a CPU-expensive thread pool. This will spawn at most N threads,
    // where N is the number of CPU cores.
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let resp = state
            .read()
            .expect("error unlocking state")
            .validator
            .validate_yaml(payload.payload());
        let _ = send.send(resp);
    });
    let resp = recv.await.expect("Panic in rayon::spawn");

    match resp {
        Ok(resp) => {
            let resp =
                serde_json::to_value(resp).expect("Validation response must be serializable");
            (status_code, response::Json(resp))
        }
        Err(e) => {
            status_code = StatusCode::BAD_REQUEST;
            (
                status_code,
                response::Json(json!({"errors": [e.to_string()]})),
            )
        }
    }
}
