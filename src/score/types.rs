//! Plain data for the tuning score: judged notes and the running per-take summary.

/// Average error at or below this counts as perfect tuning (trained singers land within ~5-10 cents).
pub const PERFECT_CENTS: f32 = 8.0;
/// Average |error| of pitches unrelated to the semitone grid (uniform over -50..50): the zero point.
pub const RANDOM_CENTS: f32 = 25.0;
/// Fewer judged notes than this give no score yet (one lucky note is not a score).
pub const MIN_SCORED_NOTES: u32 = 3;

/// A sustained note: pitch held steady long enough to judge where it sits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeldNote {
    /// Mean fractional MIDI pitch over the note (vibrato averages out).
    pub center_midi: f32,
    /// Analysis frames the note lasted (~21 ms each at 48 kHz).
    pub frames: u32,
}

impl HeldNote {
    /// Signed distance of the note's center from the nearest semitone, in cents (-50..=50).
    pub fn cents_off(&self) -> f32 {
        (self.center_midi - self.center_midi.round()) * 100.0
    }
}

/// Running result for one take (a song start or replay).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TuningSummary {
    pub notes: u32,
    /// Duration-weighted mean |cents off| over judged notes; `None` before the first note.
    pub mean_abs_cents: Option<f32>,
}

impl TuningSummary {
    /// 0..=100: 100 at <= `PERFECT_CENTS` average error, 0 at `RANDOM_CENTS` (no better than chance).
    pub fn score(&self) -> Option<u8> {
        if self.notes < MIN_SCORED_NOTES {
            return None;
        }
        let err = self.mean_abs_cents?;
        let ratio = ((RANDOM_CENTS - err) / (RANDOM_CENTS - PERFECT_CENTS)).clamp(0.0, 1.0);
        Some((ratio * 100.0).round() as u8)
    }
}
