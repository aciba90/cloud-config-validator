//! Run with
//!
//! ```not_rust
//! cargo run --bin local
//! curl --unix-socket [options...] <path> <url>
//! ```

// TODO: move this code into the crate

// TODO:
// - make it programmable
// - /run ?

#[cfg(unix)]
mod unix;

#[cfg(unix)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    unix::server().await;
}

#[cfg(not(unix))]
fn main() {
    panic!("To run in an Unix Domain Socket, unix is required");
}
