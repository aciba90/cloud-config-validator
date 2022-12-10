use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use cloud_config_validator::handlers::validate;
use serde_json::json;

#[tokio::main]
async fn main() {
    let api = Router::new()
        .route("/", get(|| async { Json(json!(["/v1"])) }))
        .route("/v1/cloud-config/validate", post(validate));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api.into_make_service())
        .await
        .unwrap();
}
