//! Held-note segmentation + tuning error, fed one pitch frame at a time (allocation-free).

use super::types::{HeldNote, TuningSummary};

/// Shortest sustained note judged (~170 ms at 48 kHz): shorter pitches are passing tones or consonants.
const MIN_NOTE_FRAMES: u32 = 8;
/// Unvoiced frames tolerated inside a note (detector misses, breathy consonants).
const MAX_GAP_FRAMES: u32 = 2;
/// Largest frame-to-frame move within a note: 6 Hz vibrato of ±50 cents moves ~0.45 semitone per frame.
const MAX_STEP_SEMITONES: f32 = 0.6;
/// Largest distance from the note's running mean: wider means a new note or a glide.
const MAX_SPREAD_SEMITONES: f32 = 0.8;

#[derive(Debug, Clone, Copy, Default)]
struct Segment {
    sum: f32,
    len: u32,
    last: f32,
    gap: u32,
}

impl Segment {
    fn mean(&self) -> f32 {
        self.sum / self.len as f32
    }

    fn accepts(&self, midi: f32) -> bool {
        self.len > 0 && (midi - self.last).abs() < MAX_STEP_SEMITONES && (midi - self.mean()).abs() < MAX_SPREAD_SEMITONES
    }

    fn push(&mut self, midi: f32) {
        self.sum += midi;
        self.len += 1;
        self.last = midi;
        self.gap = 0;
    }

    fn start(midi: f32) -> Self {
        Self { sum: midi, len: 1, last: midi, gap: 0 }
    }

    fn held_note(&self) -> Option<HeldNote> {
        (self.len >= MIN_NOTE_FRAMES).then(|| HeldNote { center_midi: self.mean(), frames: self.len })
    }
}

/// Scores how close held notes sit to exact semitones. It cannot know the song's melody, so it measures
/// tuning precision, not whether the right notes were sung.
#[derive(Debug, Clone, Default)]
pub struct TuningScorer {
    seg: Segment,
    notes: u32,
    weighted_abs_cents: f64,
    weight: u64,
}

impl TuningScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one analysis frame (`Some(fractional MIDI)` when voiced). Returns the note that just ended, if judged.
    pub fn push(&mut self, midi: Option<f32>) -> Option<HeldNote> {
        match midi {
            Some(m) if self.seg.accepts(m) => {
                self.seg.push(m);
                None
            }
            Some(m) => {
                let ended = self.close();
                self.seg = Segment::start(m);
                ended
            }
            None if self.seg.len == 0 => None,
            None => {
                self.seg.gap += 1;
                if self.seg.gap > MAX_GAP_FRAMES { self.close() } else { None }
            }
        }
    }

    /// Judge the note still being held (end of song, or mic turned off).
    pub fn finish(&mut self) -> Option<HeldNote> {
        self.close()
    }

    pub fn summary(&self) -> TuningSummary {
        TuningSummary {
            notes: self.notes,
            mean_abs_cents: (self.weight > 0).then(|| (self.weighted_abs_cents / self.weight as f64) as f32),
        }
    }

    fn close(&mut self) -> Option<HeldNote> {
        let note = self.seg.held_note();
        self.seg = Segment::default();
        let note = note?;
        self.notes += 1;
        self.weighted_abs_cents += f64::from(note.cents_off().abs()) * f64::from(note.frames);
        self.weight += u64::from(note.frames);
        Some(note)
    }
}
