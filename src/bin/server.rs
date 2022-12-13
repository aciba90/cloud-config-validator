//! Run with
//!
//! ```not_rust
//! cargo run --bin server
//! ```
use cloud_config_validator::api::create_api;

#[tokio::main]
async fn main() {
    let api = create_api();

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api.into_make_service())
        .await
        .unwrap();
}
