//! Time-synced chord names a musician enters by ear, saved on this device. Pure logic; the panel is
//! `components::chords`. Only chord symbols are kept (no lyrics, nothing taken from the song's audio).

mod chart;
mod key;
mod symbol;
mod types;

pub use key::{order_cost, smoothest_order, Key, KeyStep, MIN_KEY_MARKS};
pub use types::{Chord, ChordChart, ChordCharts, ChordMark, ChordParseError, MERGE_SECS};
