//! Note lane + phrase score: the singer's own held notes over the last few seconds, and a tuning score per
//! phrase (notes between breaths). Still reference-free: it shows where notes landed, not where they should.

use std::collections::VecDeque;

use super::types::{HeldNote, LaneBar, LaneNote, LaneView, TuningSummary, TuningTally, MIN_PHRASE_NOTES};

/// Notes kept for drawing (more than fit in any window the lane shows).
pub const LANE_NOTES: usize = 32;
/// Silence that ends a phrase (~0.5 s at 48 kHz): a breath between lines, longer than a note gap.
pub const PHRASE_GAP_FRAMES: u32 = 24;
/// Fewest semitones the lane spans, so one steady note is not drawn as a full-height block.
const MIN_SPAN: i32 = 6;

#[derive(Debug, Clone, Default)]
pub struct NoteLane {
    frame: u64,
    silent: u32,
    notes: VecDeque<LaneNote>,
    phrase: TuningTally,
    last_phrase: Option<TuningSummary>,
}

impl NoteLane {
    pub fn new() -> Self {
        Self { notes: VecDeque::with_capacity(LANE_NOTES), ..Self::default() }
    }

    /// Feed one analysis frame: whether it was voiced, and the note the scorer judged on it, if any.
    /// Returns true when what the lane shows changed (a note landed or a phrase ended).
    pub fn push(&mut self, voiced: bool, judged: Option<HeldNote>) -> bool {
        self.frame += 1;
        self.silent = if voiced { 0 } else { self.silent.saturating_add(1) };
        let mut changed = false;
        if let Some(note) = judged {
            if self.notes.len() == LANE_NOTES {
                self.notes.pop_front();
            }
            self.notes.push_back(LaneNote { note, end_frame: self.frame });
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

    /// Bars for the last `window` frames, ending at the latest frame, and the semitone range to draw.
    pub fn view(&self, window: u64) -> LaneView {
        let start = self.frame.saturating_sub(window);
        let span = window.max(1) as f32;
        let x = |frame: u64| (frame.saturating_sub(start) as f32 / span).clamp(0.0, 1.0);
        let bars: Vec<LaneBar> = self
            .notes
            .iter()
            .filter(|n| n.end_frame > start)
            .map(|n| LaneBar {
                x0: x(n.end_frame.saturating_sub(u64::from(n.note.frames))),
                x1: x(n.end_frame),
                midi: n.note.center_midi,
                cents: n.note.cents_off(),
            })
            .collect();
        if bars.is_empty() {
            return LaneView::default();
        }
        let (low, high) = bars.iter().fold((i32::MAX, i32::MIN), |(lo, hi), b| {
            let m = b.midi.round() as i32;
            (lo.min(m), hi.max(m))
        });
        let pad = (MIN_SPAN - (high - low)).max(0);
        LaneView { low: low - pad / 2, high: high + (pad - pad / 2), bars }
    }
}
