use app::score::{HeldNote, LaneWindow, Melody, NoteLane, TargetNote, TuningScorer, LANE_NOTES, PHRASE_GAP_FRAMES};

/// Seconds per analysis frame in these tests (the app uses HOP_SIZE / sample rate, ~21 ms).
const FRAME: f64 = 0.02;
const PAST: LaneWindow = LaneWindow { past: 8.0, ahead: 0.0 };

/// Scorer + lane on a frame clock, fed as the mic callback does.
struct Rig {
    scorer: TuningScorer,
    lane: NoteLane,
    t: f64,
}

impl Rig {
    fn new() -> Self {
        Self { scorer: TuningScorer::new(), lane: NoteLane::new(FRAME), t: 0.0 }
    }

    fn sing(&mut self, frames: impl IntoIterator<Item = Option<f32>>) {
        for f in frames {
            self.t += FRAME;
            let judged = self.scorer.push(f);
            self.lane.push(self.t, f.is_some(), judged);
        }
    }
}

fn phrase(cents: &[f32]) -> Vec<Option<f32>> {
    cents
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            let mut f = vec![Some(60.0 + (i % 5) as f32 + c / 100.0); 20];
            f.extend(vec![None; 4]);
            f
        })
        .collect()
}

fn breath() -> Vec<Option<f32>> {
    vec![None; PHRASE_GAP_FRAMES as usize]
}

#[test]
fn test_phrase_scores_after_a_breath_not_between_notes() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0, 2.0, -1.0]));
    assert_eq!(rig.lane.phrase_score(), None, "short gaps between notes do not end the phrase");
    rig.sing(breath());
    assert_eq!(rig.lane.phrase_score(), Some(100));
}

#[test]
fn test_each_phrase_is_scored_on_its_own() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0, 1.0]));
    rig.sing(breath());
    assert_eq!(rig.lane.phrase_score(), Some(100));
    rig.sing(phrase(&[25.0, -25.0, 25.0]));
    rig.sing(breath());
    assert_eq!(rig.lane.phrase_score(), Some(0), "the sloppy second phrase is not averaged with the first");
}

#[test]
fn test_one_note_phrase_has_no_score() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0]));
    rig.sing(breath());
    assert_eq!(rig.lane.phrase_score(), None);
}

#[test]
fn test_end_phrase_scores_the_unfinished_phrase() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0, 0.0]));
    assert!(rig.lane.end_phrase());
    assert_eq!(rig.lane.phrase_score(), Some(100));
    assert!(!rig.lane.end_phrase(), "nothing left to score");
}

#[test]
fn test_view_places_notes_in_time_and_spans_at_least_six_semitones() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0, 30.0]));
    let view = rig.lane.view(LaneWindow { past: 1.0, ahead: 0.0 }, None);
    assert_eq!(view.bars.len(), 2);
    let (a, b) = (view.bars[0], view.bars[1]);
    assert!(a.x0 < a.x1 && a.x1 <= b.x0 && b.x1 <= 1.0, "left to right in time: {a:?} {b:?}");
    assert!((b.cents - 30.0).abs() < 0.5, "keeps how far off the note was: {}", b.cents);
    assert!(view.high - view.low >= 6 && view.low <= 60 && view.high >= 61, "{view:?}");
    assert_eq!(view.now_x, None, "no look-ahead, no now marker");
    assert!(view.targets.is_empty());
}

#[test]
fn test_view_drops_notes_older_than_the_window() {
    let mut rig = Rig::new();
    rig.sing(phrase(&[0.0]));
    rig.sing(vec![None; 200]); // 4 s of silence
    assert_eq!(rig.lane.view(LaneWindow { past: 2.0, ahead: 0.0 }, None).bars.len(), 0);
    assert_eq!(rig.lane.view(PAST, None).bars.len(), 1);
}

#[test]
fn test_a_seek_back_clears_notes_from_later_in_the_song() {
    let mut rig = Rig::new();
    rig.t = 60.0;
    rig.sing(phrase(&[0.0, 0.0]));
    assert_eq!(rig.lane.view(PAST, None).bars.len(), 2);
    rig.t = 10.0; // Replay / seek back
    rig.sing([None]);
    assert!(rig.lane.view(PAST, None).bars.is_empty());
}

#[test]
fn test_paused_video_keeps_notes_in_place() {
    let mut lane = NoteLane::new(FRAME);
    lane.push(5.0, true, Some(HeldNote { center_midi: 60.0, frames: 10 }));
    let before = lane.view(PAST, None);
    for _ in 0..100 {
        lane.push(5.0, false, None); // the clock stands still while paused
    }
    assert_eq!(lane.view(PAST, None).bars, before.bars);
}

#[test]
fn test_lane_keeps_a_bounded_number_of_notes() {
    let mut lane = NoteLane::new(FRAME);
    for i in 0..LANE_NOTES + 10 {
        lane.push(i as f64 * 0.01, true, Some(HeldNote { center_midi: 60.0 + (i % 3) as f32, frames: 8 }));
    }
    assert_eq!(lane.view(LaneWindow { past: 1e9, ahead: 0.0 }, None).bars.len(), LANE_NOTES);
}

#[test]
fn test_push_reports_change_only_when_something_new_shows() {
    let mut lane = NoteLane::new(FRAME);
    assert!(!lane.push(0.0, true, None));
    assert!(lane.push(0.02, true, Some(HeldNote { center_midi: 60.0, frames: 10 })));
    let changes = (0..PHRASE_GAP_FRAMES + 5).filter(|i| lane.push(0.04 + f64::from(*i) * FRAME, false, None)).count();
    assert_eq!(changes, 1, "the phrase ends once");
}

#[test]
fn test_targets_show_ahead_of_now_in_the_singers_octave() {
    // The tune: C5 then D5 (an octave above where this singer sings)
    let melody = Melody::new([
        TargetNote { start: 0.0, end: 1.0, midi: 72 },
        TargetNote { start: 1.0, end: 3.0, midi: 74 },
    ])
    .unwrap();
    let mut lane = NoteLane::new(FRAME);
    lane.push(0.8, true, Some(HeldNote { center_midi: 60.0, frames: 30 })); // C4, 0.2..0.8 s
    let view = lane.view(LaneWindow { past: 1.0, ahead: 1.0 }, Some(&melody));
    assert_eq!(view.now_x, Some(0.5));
    assert_eq!(view.targets.len(), 2, "the note under way and the next one");
    assert_eq!(view.targets[0].midi, 60.0, "C5 drawn at C4, beside the singer");
    assert_eq!(view.targets[1].midi, 62.0);
    assert!(view.targets[1].x0 >= 0.5, "the next note starts at or after now");
    assert!(view.low <= 60 && view.high >= 62);
}

#[test]
fn test_targets_alone_still_draw_at_their_own_octave() {
    let melody = Melody::new([TargetNote { start: 0.0, end: 2.0, midi: 67 }]).unwrap();
    let mut lane = NoteLane::new(FRAME);
    lane.push(0.5, false, None);
    let view = lane.view(LaneWindow { past: 1.0, ahead: 1.0 }, Some(&melody));
    assert_eq!(view.targets.len(), 1);
    assert_eq!(view.targets[0].midi, 67.0);
    assert!(view.bars.is_empty());
}
