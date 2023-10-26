#[doc = include_str!("../README.md")]
pub mod api;
pub mod error; // only public for benches
pub use ccv_core::schema;
pub use ccv_core::validator; // only public for benches
