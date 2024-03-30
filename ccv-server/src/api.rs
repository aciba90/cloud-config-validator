use std::{sync::Arc, time::Duration};

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use ccv_core::schema::ConfigKind;
use serde_json::json;
use std::sync::RwLock;
use tokio::time;
use tower_http::trace::TraceLayer;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response;
use axum::response::IntoResponse;

use crate::error::ApiError;
use crate::validator::CloudConfig;
use crate::validator::Validator;

#[derive(Debug)]
pub struct AppState {
    pub cc_validator: Validator,
    pub nc_validator: Validator,
}

#[tracing::instrument(level = "info", skip(state, payload))] // do not leak the payload
pub async fn validate(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<CloudConfig>,
) -> Result<impl IntoResponse, ApiError> {
    // Note: `validate_yaml` over an unbound yaml could block the async runtime, causing a delay
    // in response time.
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
            .cc_validator
            .validate_yaml(payload.payload());
        let _ = send.send(resp);
    });
    let resp = recv.await.expect("Panic in rayon::spawn")?;
    let resp = serde_json::to_value(resp).expect("Validation response must be serializable");
    Ok((StatusCode::OK, response::Json(resp)))
}

#[tracing::instrument(level = "info", skip(state, payload))] // do not leak the payload
pub async fn nc_validate(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<CloudConfig>,
) -> Result<impl IntoResponse, ApiError> {
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let resp = state
            .read()
            .expect("error unlocking state")
            .nc_validator
            .validate_yaml(payload.payload());
        let _ = send.send(resp);
    });
    let resp = recv.await.expect("Panic in rayon::spawn")?;
    let resp = serde_json::to_value(resp).expect("Validation response must be serializable");
    Ok((StatusCode::OK, response::Json(resp)))
}

pub async fn create_api() -> Router {
    let cc_validator = match Validator::new(ConfigKind::CloudConfig).await {
        Err(e) => panic!("Error reading the JsonSchema: {}", e),
        Ok(v) => v,
    };
    let nc_validator = match Validator::new(ConfigKind::NetworkConfig).await {
        Err(e) => panic!("Error reading the JsonSchema: {}", e),
        Ok(v) => v,
    };
    let app_state = AppState {
        cc_validator,
        nc_validator,
    };
    let shared_state = Arc::new(RwLock::new(app_state));

    // spawn job to refresh the schema periodically
    tokio::spawn({
        // TODO: make interval period configurable
        // TODO: skip first tick
        let mut interval = time::interval(Duration::from_secs(60 * 60)); // 1 hour
        let shared_state = shared_state.clone();
        async move {
            loop {
                interval.tick().await;
                tracing::info!("refreshing cloud-config jsonschema");
                let cc_validator = Validator::new(ConfigKind::CloudConfig).await;
                match cc_validator {
                    Err(e) => {
                        tracing::error!(
                            "Error reading new JsonSchema. Re-using the previous one: {}",
                            e
                        )
                    }
                    Ok(validator) => {
                        shared_state
                            .write()
                            .expect("Error locking `ApiState`")
                            .cc_validator = validator
                    }
                };

                tracing::info!("refreshing network-config jsonschema");
                let nc_validator = Validator::new(ConfigKind::NetworkConfig).await;
                match nc_validator {
                    Err(e) => {
                        tracing::error!(
                            "Error reading new JsonSchema. Re-using the previous one: {}",
                            e
                        )
                    }
                    Ok(validator) => {
                        shared_state
                            .write()
                            .expect("Error locking `ApiState`")
                            .nc_validator = validator
                    }
                };
            }
        }
    });

    Router::new()
        .route("/", get(|| async { Json(json!(["/v1"])) }))
        .route("/v1/cloud-config/validate", post(validate))
        .route("/v1/network-config/validate", post(nc_validate))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state)
}

#[cfg(test)]
mod test {
    use super::*;
    use axum_test::TestServer;

    async fn test_client() -> TestServer {
        let api = create_api().await;

        TestServer::new(api).unwrap()
    }

    #[tokio::test]
    async fn root() {
        let client = test_client().await;
        let res = client.get("/").await;
        assert_eq!(res.status_code(), StatusCode::OK);
        assert_eq!(res.text(), "[\"/v1\"]");
    }

    #[tokio::test]
    async fn invalid_yaml() {
        let client = test_client().await;
        let res = client
            .post("/v1/cloud-config/validate")
            .json(&json!({"payload": "\"a"}))
            .await;
        assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(res.text(), "{\"errors\":[\"found unexpected end of stream at line 1 column 3, while scanning a quoted scalar\"]}");
    }

    #[tokio::test]
    async fn nc_valid_yaml() {
        let client = test_client().await;
        let network_config = r#"
network:
  version: 1
  config:
  - type: bond
    name: a
    mac_address: aa:bb
    mtu: 1
    subnets:
    - type: dhcp6
      control: manual
      netmask: 255.255.255.0
      gateway: 10.0.0.1
      dns_nameservers:
      - 8.8.8.8
      dns_search:
      - find.me
      routes:
      - type: route
        destination: 10.20.0.0/8
        gateway: a.b.c.d
        metric: 200"#;
        let res = client
            .post("/v1/network-config/validate")
            .json(&json!({"payload": network_config}))
            .await;
        assert_eq!(res.status_code(), StatusCode::OK);
        assert_eq!(
            res.text(),
            "{\"annotations\":[],\"errors\":[],\"is_valid\":true}"
        );
    }
}
