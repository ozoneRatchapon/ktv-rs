//! Plain data for the tuning score: judged notes and the running per-take summary.

/// Average error at or below this counts as perfect tuning (trained singers land within ~5-10 cents).
pub const PERFECT_CENTS: f32 = 8.0;
/// Average |error| of pitches unrelated to the semitone grid (uniform over -50..50): the zero point.
pub const RANDOM_CENTS: f32 = 25.0;
/// Fewer judged notes than this give no score yet (one lucky note is not a score).
pub const MIN_SCORED_NOTES: u32 = 3;
/// A phrase is short: two held notes are enough to judge it.
pub const MIN_PHRASE_NOTES: u32 = 2;

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
        self.score_after(MIN_SCORED_NOTES)
    }

    /// Same scale as `score`, given once at least `min_notes` notes were judged.
    pub fn score_after(&self, min_notes: u32) -> Option<u8> {
        if self.notes < min_notes {
            return None;
        }
        let err = self.mean_abs_cents?;
        let ratio = ((RANDOM_CENTS - err) / (RANDOM_CENTS - PERFECT_CENTS)).clamp(0.0, 1.0);
        Some((ratio * 100.0).round() as u8)
    }
}

/// Running duration-weighted tally of judged notes (a whole take, or one phrase).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TuningTally {
    notes: u32,
    weighted_abs_cents: f64,
    weight: u64,
}

impl TuningTally {
    pub fn add(&mut self, note: &HeldNote) {
        self.notes += 1;
        self.weighted_abs_cents += f64::from(note.cents_off().abs()) * f64::from(note.frames);
        self.weight += u64::from(note.frames);
    }

    pub fn summary(&self) -> TuningSummary {
        TuningSummary {
            notes: self.notes,
            mean_abs_cents: (self.weight > 0).then(|| (self.weighted_abs_cents / self.weight as f64) as f32),
        }
    }
}

/// A judged note placed in time for the note lane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneNote {
    pub note: HeldNote,
    /// Clock time (song seconds) where the note ended.
    pub end_t: f64,
}

/// One note bar of the lane, ready to draw: `x0..x1` in 0..=1 across the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneBar {
    pub x0: f32,
    pub x1: f32,
    pub midi: f32,
    pub cents: f32,
}

/// What the lane shows: the singer's bars and the melody's target bars inside the time window, and the whole
/// semitones spanned (`low..=high`). `now_x` marks the present when the window also shows what is coming.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LaneView {
    pub low: i32,
    pub high: i32,
    pub bars: Vec<LaneBar>,
    pub targets: Vec<LaneBar>,
    pub now_x: Option<f32>,
}

/// One note of a song's melody (the tune the singer should hit), in karaoke-video seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetNote {
    pub start: f32,
    pub end: f32,
    /// MIDI note number (60 = C4).
    pub midi: u8,
}

/// Running melody result for one take: frames where the song had a target note, and how many of them the
/// singer was on the right note (any octave).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MelodySummary {
    pub target_frames: u32,
    pub hit_frames: u32,
}
