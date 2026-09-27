//! Melody score: does the singer hit the song's notes? Needs per-song melody data (`ktv.melodies.v1`), which no
//! build ships: where it comes from is an owner decision (plan 002 item 7). Octave-tolerant: a note sung an octave
//! (or two) away from the target counts, so any voice can score.

use std::collections::HashMap;

use super::types::{MelodySummary, TargetNote};

/// Notes kept per song (a 5-minute song has a few hundred).
pub const MAX_MELODY_NOTES: usize = 5000;
/// The singer hears the video late (output latency) and answers late (reaction): the voice at song time `t`
/// is compared with the melody at `t - SING_LAG_SECS`. Checklist 8 (latency calibration) would measure it per device.
pub const SING_LAG_SECS: f64 = 0.15;
/// A note sung this close to a target's start or end still counts for that target (early / late entries).
pub const EDGE_SECS: f64 = 0.1;
/// Nearest semitone must be the target's pitch class: |octave-folded error| at most half a semitone.
pub const HIT_CENTS: f32 = 50.0;
/// Coverage that scores 100: breaths and consonants inside long notes make every frame impossible.
pub const FULL_COVERAGE: f32 = 0.8;
/// Target frames needed before a melody score is shown (~2 s of the tune at 47 frames/s).
pub const MIN_TARGET_FRAMES: u32 = 90;
/// The clock standing still for this many frames in a row (~0.1 s) means the video is paused: frames are not judged.
/// Not fewer: mic frames reach the page in bursts, and frames handled in the same millisecond read the same clock.
pub const PAUSED_FRAMES: u32 = 5;

/// A song's melody, sorted by start, validated (finite times, `end > start`, MIDI 24..=96, at most
/// `MAX_NOTE_SECS` long).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Melody {
    notes: Vec<TargetNote>,
}

impl Melody {
    /// Keep the valid notes (sorted); `None` if none are left.
    pub fn new(notes: impl IntoIterator<Item = TargetNote>) -> Option<Self> {
        let mut notes: Vec<TargetNote> = notes
            .into_iter()
            .filter(|n| n.start.is_finite() && n.end.is_finite() && n.start >= 0.0 && n.end > n.start && (24..=96).contains(&n.midi))
            .take(MAX_MELODY_NOTES)
            .map(|n| TargetNote { end: n.end.min(n.start + MAX_NOTE_SECS as f32), ..n })
            .collect();
        notes.sort_by(|a, b| a.start.total_cmp(&b.start));
        (!notes.is_empty()).then_some(Self { notes })
    }

    pub fn notes(&self) -> &[TargetNote] {
        &self.notes
    }

    /// Notes sounding at any moment in `from..=to` (seconds).
    pub fn between(&self, from: f64, to: f64) -> impl Iterator<Item = &TargetNote> {
        // Notes are sorted by start; the longest note bounds how far back an overlapping one can begin
        let first = self.notes.partition_point(|n| f64::from(n.start) < from - MAX_NOTE_SECS);
        self.notes[first..].iter().take_while(move |n| f64::from(n.start) <= to).filter(move |n| f64::from(n.end) >= from)
    }

    /// The note sounding at `t`, if any.
    pub fn at(&self, t: f64) -> Option<&TargetNote> {
        self.between(t, t).next()
    }
}

/// Notes longer than this are cut when drawn and matched (keeps `between` a bounded scan).
const MAX_NOTE_SECS: f64 = 30.0;

/// Signed distance from `sung` (fractional MIDI) to the nearest octave of `target`, in cents (-600..=600).
pub fn octave_cents(sung: f32, target: u8) -> f32 {
    let semis = (sung - f32::from(target)).rem_euclid(12.0);
    let folded = if semis > 6.0 { semis - 12.0 } else { semis };
    folded * 100.0
}

/// Per-take melody score, fed one analysis frame at a time with the song time it was heard at.
#[derive(Debug, Clone, Default)]
pub struct MelodyScorer {
    summary: MelodySummary,
    last_t: Option<f64>,
    /// Frames in a row that read the same clock value.
    still: u32,
}

impl MelodyScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Judge one frame: the song time `t` and the sung pitch (`None` when silent). Returns true if the summary changed.
    pub fn push(&mut self, melody: &Melody, t: f64, midi: Option<f32>) -> bool {
        self.still = if self.last_t == Some(t) { self.still + 1 } else { 0 };
        self.last_t = Some(t);
        let paused = self.still >= PAUSED_FRAMES;
        let heard_at = t - SING_LAG_SECS;
        if paused || melody.at(heard_at).is_none() {
            return false;
        }
        self.summary.target_frames += 1;
        let hit = midi.is_some_and(|m| {
            melody.between(heard_at - EDGE_SECS, heard_at + EDGE_SECS).any(|n| octave_cents(m, n.midi).abs() <= HIT_CENTS)
        });
        if hit {
            self.summary.hit_frames += 1;
        }
        true
    }

    pub fn summary(&self) -> MelodySummary {
        self.summary
    }
}

impl MelodySummary {
    /// 0..=100: the share of the tune's notes sung on the right note, 100 at [`FULL_COVERAGE`]. `None` until
    /// [`MIN_TARGET_FRAMES`] of the tune were heard.
    pub fn score(&self) -> Option<u8> {
        if self.target_frames < MIN_TARGET_FRAMES {
            return None;
        }
        let coverage = self.hit_frames as f32 / self.target_frames as f32;
        Some(((coverage / FULL_COVERAGE).min(1.0) * 100.0).round() as u8)
    }
}

/// Stored melodies by song id: `{ "<song id>": [[start, end, midi], ...] }` (compact: thousands of notes fit).
pub type StoredMelodies = HashMap<String, Vec<(f32, f32, u8)>>;

/// The melody stored for `song_id`, validated; `None` if there is none or nothing valid in it.
pub fn stored_melody(store: &StoredMelodies, song_id: &str) -> Option<Melody> {
    let notes = store.get(song_id)?;
    Melody::new(notes.iter().map(|&(start, end, midi)| TargetNote { start, end, midi }))
}

