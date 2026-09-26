//! Monophonic pitch detection for the singer's microphone.

mod gate;
mod mpm;
mod types;

pub use gate::{rms, NoiseGate, GATE_RATIO, MAX_GATE_RMS, MIN_GATE_RMS, ROOM_CHECK_FRAMES};
pub use mpm::Mpm;
pub use types::{MpmConfig, NoteReading, PitchEstimate};
