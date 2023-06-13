#[doc = include_str!("../README.md")]
pub mod api;
pub mod error; // only public for benches
mod schema;
pub mod validator; // only public for benches
