use std::collections::BTreeMap;

use crate::types::GuideTrack;

/// A moment the curator judged in sync by ear: this karaoke second lines up with this MV second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyncMark {
    pub karaoke_secs: f64,
    pub mv_secs: f64,
}

/// Why two marks cannot give a guide mapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitError {
    /// Marks closer than [`super::MIN_FIT_SPAN_SECS`] of karaoke time: rate would be mostly ear error.
    TooClose { span_secs: f64 },
    /// Fitted speed outside [`super::RATE_RANGE`]: the MV is another edit or a mark was wrong.
    RateOutOfRange { rate: f64 },
}

impl std::fmt::Display for FitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooClose { span_secs } => {
                let min = super::MIN_FIT_SPAN_SECS;
                write!(f, "marks are {span_secs:.0}s apart; put them at least {min:.0}s apart (early verse + late chorus)")
            }
            Self::RateOutOfRange { rate } => {
                write!(f, "fitted speed {rate:.3} is not plausible; the MV may be a different edit, or re-mark")
            }
        }
    }
}

impl std::error::Error for FitError {}

/// Guide timings set in the app, keyed by song id; they win over `assets/catalog.json`.
pub type GuideOverrides = BTreeMap<String, GuideTrack>;
