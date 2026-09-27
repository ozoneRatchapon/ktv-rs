//! A song's beat grid, tapped in by a musician (no tempo data ships; nothing is taken from the song's audio).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Beats at `anchor + k * period` karaoke seconds, for every whole `k`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BeatGrid {
    pub period: f64,
    pub anchor: f64,
}

/// Saved grids by song id (`ktv.tempo.v1`).
pub type BeatGrids = HashMap<String, BeatGrid>;

/// When the count-in clicks, and where playback starts so the player is running before the first click.
#[derive(Debug, Clone, PartialEq)]
pub struct CountIn {
    /// Karaoke second to seek to.
    pub start: f64,
    /// Karaoke seconds of the clicks, earliest first; the beat after the last one is where the part begins.
    pub clicks: Vec<f64>,
    /// The beat the part begins on (the loop's A, moved to the nearest beat).
    pub land: f64,
}
