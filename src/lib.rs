#[doc = include_str!("../README.md")]
pub mod api;
pub mod error; // only public for benches
mod handlers;
mod schema;
mod state;
pub mod telemetry;
pub mod validator; // only public for benches
