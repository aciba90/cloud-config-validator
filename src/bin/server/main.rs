//! Run with
//!
//! ```not_rust
//! cargo run --bin server
//! ```
mod telemetry;

use cloud_config_validator::api::create_api;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let subscriber = telemetry::get_subscriber();
    telemetry::init_subscriber(subscriber);
    let api = create_api().await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(api.into_make_service())
        .await
        .unwrap();
}
