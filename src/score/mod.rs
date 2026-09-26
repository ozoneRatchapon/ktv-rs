//! Singing score without a reference melody (no licensed note data needed).

mod history;
mod tuning;
mod types;

pub use history::{record, TakeResult, MAX_RESULTS};
pub use tuning::TuningScorer;
pub use types::{HeldNote, TuningSummary, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS};
