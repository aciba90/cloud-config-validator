use std::sync::Arc;

use crate::{handlers::validate, state::AppState, validator::Validator};
use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;

pub async fn create_api() -> Router {
    let validator = Validator::new().await;
    let app_state = AppState { validator };
    let shared_state = Arc::new(app_state);

    Router::new()
        .route("/", get(|| async { Json(json!(["/v1"])) }))
        .route("/v1/cloud-config/validate", post(validate))
        .with_state(shared_state)
}

#[cfg(test)]
mod test {
    use super::*;
    use axum_test_helper::TestClient;
    use hyper::StatusCode;

    async fn test_client() -> TestClient {
        let api = create_api().await;

        TestClient::new(api)
    }

    #[tokio::test]
    async fn root() {
        let client = test_client().await;
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "[\"/v1\"]");
    }

    #[tokio::test]
    async fn invalid_yaml() {
        let client = test_client().await;
        let res = client
            .post("/v1/cloud-config/validate")
            .json(&json!({"payload": "\"a"}))
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(res.text().await, "{\"errors\":[\"found unexpected end of stream at line 1 column 3, while scanning a quoted scalar\"]}");
    }
}
