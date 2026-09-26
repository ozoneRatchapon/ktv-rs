//! Monophonic pitch detection for the singer's microphone.

mod mpm;
mod types;

pub use mpm::Mpm;
pub use types::{MpmConfig, NoteReading, PitchEstimate};
