//! The booth's play state: current song, queue and queue ids. Pure logic; `main.rs` holds it in one signal.

mod ops;
mod types;

pub use ops::KEY_RANGE;
pub use types::{Booth, Placement, Requester};
