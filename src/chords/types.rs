use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A mark this close to an existing one replaces it (a re-tap of the same change, not a new chord).
pub const MERGE_SECS: f64 = 0.3;

/// A parsed chord symbol, e.g. `F#m7/C#`: root and bass as pitch classes (0 = C), the quality suffix in its
/// canonical spelling, and whether to spell accidentals as flats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    pub root: u8,
    pub quality: &'static str,
    pub bass: Option<u8>,
    pub flats: bool,
}

/// Why a typed chord name was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordParseError {
    Empty,
    /// The root must be a note letter A–G.
    Root,
    /// Not a known chord quality (m, 7, maj7, sus4, dim ...).
    Quality,
    /// The part after `/` must be a note.
    Bass,
}

/// One chord change: from `at_secs` of karaoke time until the next mark. `chord` is canonical (see [`Chord`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChordMark {
    pub at_secs: f64,
    pub chord: String,
}

/// A song's chord changes, sorted by time.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChordChart {
    pub(super) marks: Vec<ChordMark>,
}

/// Chord charts on this device, by song id (`crate::storage::CHORDS_KEY`).
pub type ChordCharts = BTreeMap<String, ChordChart>;
