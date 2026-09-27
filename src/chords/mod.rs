//! Time-synced chord names a musician enters by ear, saved on this device. Pure logic; the panel is
//! `components::chords`. Only chord symbols are kept (no lyrics, nothing taken from the song's audio).

mod chart;
mod symbol;
mod types;

pub use types::{Chord, ChordChart, ChordCharts, ChordMark, ChordParseError, MERGE_SECS};
