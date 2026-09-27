//! Singing score without a reference melody (no licensed note data needed).

mod history;
mod lane;
mod party;
mod tuning;
mod types;

pub use history::{record, TakeResult, MAX_RESULTS};
pub use lane::{NoteLane, LANE_NOTES, PHRASE_GAP_FRAMES};
pub use party::{clean_name, leaderboard, name_take, recent_singers, LeaderRow, MAX_NAME_CHARS, PARTY_WINDOW_MS};
pub use tuning::TuningScorer;
pub use types::{HeldNote, LaneBar, LaneView, TuningSummary, MIN_PHRASE_NOTES, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS};
