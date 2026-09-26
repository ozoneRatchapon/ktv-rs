//! Singer's microphone: capture in an AudioWorklet, analysis frames delivered to Rust.

mod types;
#[cfg(not(target_arch = "wasm32"))]
mod unsupported;
#[cfg(target_arch = "wasm32")]
mod web;

pub use types::{MicError, FRAME_SIZE, HOP_SIZE};
#[cfg(not(target_arch = "wasm32"))]
pub use unsupported::Mic;
#[cfg(target_arch = "wasm32")]
pub use web::Mic;
