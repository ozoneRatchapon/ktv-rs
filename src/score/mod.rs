//! Singing score without a reference melody (no licensed note data needed).

mod tuning;
mod types;

pub use tuning::TuningScorer;
pub use types::{HeldNote, TuningSummary, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS};
