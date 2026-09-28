//! Finished takes, newest first, kept per device.

use serde::{Deserialize, Serialize};

use super::types::TuningSummary;

/// Takes kept in the history (older ones drop off).
/// Enough for a long party night (a take is ~150 bytes in storage).
pub const MAX_RESULTS: usize = 100;

/// One finished take the mic listened to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakeResult {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub notes: u32,
    /// Duration-weighted mean |cents off| over judged notes.
    pub mean_abs_cents: f32,
    /// Who sang, if someone typed it on the result card (stays on this device).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub singer: Option<String>,
    /// When the take ended (ms since the epoch); 0 for takes saved before this was kept.
    #[serde(default)]
    pub sung_at_ms: f64,
    /// Melody score of the take, when the song had melody data on this device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melody_score: Option<u8>,
    /// Duet on a stereo receiver: which singer (1 = left mic, 2 = right); `None` for a solo take.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<u8>,
    /// A medley part's take: which medley and where in it; `None` for a whole song.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medley: Option<MedleyTake>,
}

/// Where a take sits in a medley (plan 004 M5b).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MedleyTake {
    /// Queue id of the medley's first part: the same for every part of one queued medley.
    pub id: u64,
    /// 0-based position in the medley.
    pub index: u32,
    pub count: u32,
    pub title: String,
}

impl TakeResult {
    /// `None` when no held note was judged (mic on but silent): nothing worth keeping.
    pub fn new(song_id: &str, title: &str, artist: &str, summary: TuningSummary, sung_at_ms: f64) -> Option<Self> {
        let mean_abs_cents = summary.mean_abs_cents?;
        Some(Self {
            song_id: song_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            notes: summary.notes,
            mean_abs_cents,
            singer: None,
            sung_at_ms,
            melody_score: None,
            part: None,
            medley: None,
        })
    }

    pub fn with_part(self, part: Option<u8>) -> Self {
        Self { part, ..self }
    }

    pub fn with_medley(self, medley: Option<MedleyTake>) -> Self {
        Self { medley, ..self }
    }

    pub fn with_melody_score(self, melody_score: Option<u8>) -> Self {
        Self { melody_score, ..self }
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
