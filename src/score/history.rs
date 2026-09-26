//! Finished takes, newest first, kept per device.

use serde::{Deserialize, Serialize};

use super::types::TuningSummary;

/// Takes kept in the history (older ones drop off).
pub const MAX_RESULTS: usize = 20;

/// One finished take the mic listened to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakeResult {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub notes: u32,
    /// Duration-weighted mean |cents off| over judged notes.
    pub mean_abs_cents: f32,
}

impl TakeResult {
    /// `None` when no held note was judged (mic on but silent): nothing worth keeping.
    pub fn new(song_id: &str, title: &str, artist: &str, summary: TuningSummary) -> Option<Self> {
        let mean_abs_cents = summary.mean_abs_cents?;
        Some(Self {
            song_id: song_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            notes: summary.notes,
            mean_abs_cents,
        })
    }

    pub fn summary(&self) -> TuningSummary {
        TuningSummary { notes: self.notes, mean_abs_cents: Some(self.mean_abs_cents) }
    }

    pub fn score(&self) -> Option<u8> {
        self.summary().score()
    }
}

/// Add a take at the front, keeping at most [`MAX_RESULTS`].
pub fn record(history: &mut Vec<TakeResult>, result: TakeResult) {
    history.insert(0, result);
    history.truncate(MAX_RESULTS);
}
