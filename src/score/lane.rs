//! Note lane + phrase score: the singer's held notes over the last few seconds on the song's clock, and a tuning
//! score per phrase (notes between breaths). With a melody, the tune's notes are drawn behind them, a little ahead.

use std::collections::VecDeque;

use super::melody::Melody;
use super::types::{HeldNote, LaneBar, LaneNote, LaneView, TuningSummary, TuningTally, MIN_PHRASE_NOTES};

/// Notes kept for drawing (more than fit in any window the lane shows).
pub const LANE_NOTES: usize = 32;
/// Silence that ends a phrase (~0.5 s at 48 kHz): a breath between lines, longer than a note gap.
pub const PHRASE_GAP_FRAMES: u32 = 24;
/// Fewest semitones the lane spans, so one steady note is not drawn as a full-height block.
const MIN_SPAN: i32 = 6;
/// The clock jumping back more than this is a seek (or Replay): notes drawn from later in the song are dropped.
const SEEK_BACK_SECS: f64 = 1.0;

/// The part of the song the lane shows: `past` seconds before now, and `ahead` seconds after (targets only).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneWindow {
    pub past: f64,
    pub ahead: f64,
}

#[derive(Debug, Clone, Default)]
pub struct NoteLane {
    /// Seconds per analysis frame (a held note's length in frames becomes its length in song time).
    frame_secs: f64,
    now: f64,
    silent: u32,
    notes: VecDeque<LaneNote>,
    phrase: TuningTally,
    last_phrase: Option<TuningSummary>,
}

impl NoteLane {
    pub fn new(frame_secs: f64) -> Self {
        Self { frame_secs, notes: VecDeque::with_capacity(LANE_NOTES), ..Self::default() }
    }

    /// Feed one analysis frame heard at clock time `t` (song seconds): whether it was voiced, and the note the
    /// scorer judged on it, if any. Returns true when what the lane shows changed (a note landed or a phrase ended).
    pub fn push(&mut self, t: f64, voiced: bool, judged: Option<HeldNote>) -> bool {
        let mut changed = false;
        if t < self.now - SEEK_BACK_SECS && !self.notes.is_empty() {
            self.notes.clear();
            changed = true;
        }
        self.now = t;
        self.silent = if voiced { 0 } else { self.silent.saturating_add(1) };
        if let Some(note) = judged {
            if self.notes.len() == LANE_NOTES {
                self.notes.pop_front();
            }
            self.notes.push_back(LaneNote { note, end_t: t });
            self.phrase.add(&note);
            changed = true;
        }
        if self.silent == PHRASE_GAP_FRAMES {
            changed |= self.end_phrase();
        }
        changed
    }

    /// Close the phrase being sung (end of the take). Returns true if it was scored.
    pub fn end_phrase(&mut self) -> bool {
        let phrase = std::mem::take(&mut self.phrase).summary();
        match phrase.notes {
            0 => false,
            _ => {
                self.last_phrase = Some(phrase);
                true
            }
        }
    }

    /// Tuning score of the last finished phrase (same 0-100 scale as the take).
    pub fn phrase_score(&self) -> Option<u8> {
        self.last_phrase?.score_after(MIN_PHRASE_NOTES)
    }

    /// Bars for `window` around the latest clock time, the melody's notes in it (moved to the octave the singer
    /// is using, so a low voice sees the tune at its own height), and the semitone range to draw.
    pub fn view(&self, window: LaneWindow, melody: Option<&Melody>) -> LaneView {
        let start = self.now - window.past;
        let end = self.now + window.ahead;
        let span = (end - start).max(f64::EPSILON);
        let x = |t: f64| ((t - start) / span).clamp(0.0, 1.0) as f32;
        let bars: Vec<LaneBar> = self
            .notes
            .iter()
            .map(|n| (n, n.end_t - f64::from(n.note.frames) * self.frame_secs))
            .filter(|(n, from)| n.end_t > start && *from < self.now)
            .map(|(n, from)| LaneBar { x0: x(from), x1: x(n.end_t), midi: n.note.center_midi, cents: n.note.cents_off() })
            .collect();
        let targets: Vec<LaneBar> = melody
            .map(|m| {
                m.between(start, end)
                    .map(|n| LaneBar {
                        x0: x(f64::from(n.start)),
                        x1: x(f64::from(n.end)),
                        midi: f32::from(n.midi),
                        cents: 0.0,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let targets = to_singers_octave(targets, &bars);
        let Some((low, high)) = bars.iter().chain(&targets).fold(None, |range: Option<(i32, i32)>, b| {
            let m = b.midi.round() as i32;
            Some(range.map_or((m, m), |(lo, hi)| (lo.min(m), hi.max(m))))
        }) else {
            return LaneView::default();
        };
        let pad = (MIN_SPAN - (high - low)).max(0);
        let now_x = (window.ahead > 0.0).then(|| x(self.now));
        LaneView { low: low - pad / 2, high: high + (pad - pad / 2), bars, targets, now_x }
    }
}

/// Shift target bars by whole octaves toward the singer's average pitch (no singer bars: leave them).
fn to_singers_octave(mut targets: Vec<LaneBar>, bars: &[LaneBar]) -> Vec<LaneBar> {
    if targets.is_empty() || bars.is_empty() {
        return targets;
    }
    let mean = |v: &[LaneBar]| v.iter().map(|b| b.midi).sum::<f32>() / v.len() as f32;
    let octaves = ((mean(bars) - mean(&targets)) / 12.0).round();
    for t in &mut targets {
        t.midi += octaves * 12.0;
    }
    targets
}
