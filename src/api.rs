use std::{sync::Arc, time::Duration};

use crate::{handlers::validate, state::AppState, validator::Validator};
use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
};
use axum::{Extension, Json};
use serde_json::json;
use std::sync::RwLock;
use tokio::time;

// TODO: this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api_doc(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

pub async fn create_api() -> ApiRouter {
    let validator = Validator::new().await;
    let app_state = AppState { validator };
    let shared_state = Arc::new(RwLock::new(app_state));

    // TODO: make interval period configurable
    // TODO: do not refresh at the begining
    // spawn job to refresh the schema periodically
    tokio::spawn({
        let mut interval = time::interval(Duration::from_secs(60 * 60)); // 1 hour
        let shared_state = shared_state.clone();
        async move {
            loop {
                interval.tick().await;
                // TODO: log event
                println!("refreshing cloud-config jsonschema");
                let validator = Validator::new().await;
                shared_state.write().unwrap().validator = validator;
            }
        }
    });

    ApiRouter::new()
        .api_route("/", get(|| async { Json(json!(["/v1"])) }))
        .api_route("/v1/cloud-config/validate", post(validate))
        .route("/api.json", get(serve_api_doc))
        .with_state(shared_state)
}

pub fn get_api_doc() -> OpenApi {
    OpenApi {
        info: Info {
            description: Some("asdfaf".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    }
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
