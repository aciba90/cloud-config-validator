use crate::handlers::validate;
use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;

pub fn create_api() -> Router {
    Router::new()
        .route("/", get(|| async { Json(json!(["/v1"])) }))
        .route("/v1/cloud-config/validate", post(validate))
}

#[cfg(test)]
mod test {
    use super::*;
    use axum_test_helper::TestClient;
    use hyper::StatusCode;

    fn test_client() -> TestClient {
        let api = create_api();

        TestClient::new(api)
    }

    #[tokio::test]
    async fn root() {
        let client = test_client();
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "[\"/v1\"]");
    }

    #[tokio::test]
    async fn invalid_yaml() {
        let client = test_client();
        let res = client
            .post("/v1/cloud-config/validate")
            .json(&json!({"payload": "\"a"}))
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(res.text().await, "{\"errors\":[\"found unexpected end of stream at line 1 column 3, while scanning a quoted scalar\"]}");
    }
}
