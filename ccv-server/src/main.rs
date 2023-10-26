//! Run with
//!
//! ```not_rust
//! cargo run --bin server
//! ```
mod telemetry {
    use tracing::{subscriber::set_global_default, Subscriber};
    use tracing_subscriber::layer::SubscriberExt;

    /// Compose multiple layers into a `tracing`'s subscriber.
    pub fn get_subscriber() -> impl Subscriber + Send + Sync {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "cloud_config_validator=trace,tower_http=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
    }

    /// Register a subscriber as global default to process span data.
    ///
    /// It should only be called once!
    pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
        set_global_default(subscriber).expect("Failed to set subscriber");
    }
}

use ccv_server::api::create_api;
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
