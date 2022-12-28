//! Run with
//!
//! ```not_rust
//! cargo run --bin server
//! ```
use axum::Extension;
use cloud_config_validator::api::{create_api, get_api_doc};

#[tokio::main]
async fn main() {
    let api = create_api().await;
    let mut api_doc = get_api_doc();

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            api.finish_api(&mut api_doc)
                .layer(Extension(api_doc))
                .into_make_service(),
        )
        .await
        .unwrap();
}
