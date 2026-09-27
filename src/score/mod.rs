//! Singing scores: tuning (no reference melody needed) and, when a song has melody data, melody.

mod history;
mod lane;
mod melody;
mod party;
mod tuning;
mod types;

pub use history::{record, TakeResult, MAX_RESULTS};
pub use lane::{LaneWindow, NoteLane, LANE_NOTES, PHRASE_GAP_FRAMES};
pub use melody::{
    octave_cents, stored_melody, Melody, MelodyScorer, StoredMelodies, EDGE_SECS, FULL_COVERAGE, HIT_CENTS, MIN_TARGET_FRAMES,
    PAUSED_FRAMES, SING_LAG_SECS,
};
pub use party::{clean_name, leaderboard, name_take, recent_singers, LeaderRow, MAX_NAME_CHARS, PARTY_WINDOW_MS};
pub use tuning::TuningScorer;
pub use types::{HeldNote, LaneBar, LaneView, MelodySummary, TargetNote, TuningSummary, MIN_PHRASE_NOTES, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS};
