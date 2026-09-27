//! Count-in for musician mode: a beat grid tapped along with the song, saved on this device, and the clicks
//! that lead into a practice part. Pure logic; the buttons are `components::tempo`.

mod fit;
mod types;

pub use fit::{add_tap, fit, sanitize, COUNT_IN_BEATS, MAX_BPM, MIN_BPM, MIN_TAPS, PRE_ROLL_SECS, TAP_GAP_SECS};
pub use types::{BeatGrid, BeatGrids, CountIn};
