use app::score::{HeldNote, NoteLane, TuningScorer, LANE_NOTES, PHRASE_GAP_FRAMES};

/// Feeds frames through the scorer into the lane, as the mic callback does.
fn sing(scorer: &mut TuningScorer, lane: &mut NoteLane, frames: impl IntoIterator<Item = Option<f32>>) {
    for f in frames {
        let judged = scorer.push(f);
        lane.push(f.is_some(), judged);
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
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0, 2.0, -1.0]));
    assert_eq!(lane.phrase_score(), None, "short gaps between notes do not end the phrase");
    sing(&mut s, &mut lane, breath());
    assert_eq!(lane.phrase_score(), Some(100));
}

#[test]
fn test_each_phrase_is_scored_on_its_own() {
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0, 1.0]));
    sing(&mut s, &mut lane, breath());
    assert_eq!(lane.phrase_score(), Some(100));
    sing(&mut s, &mut lane, phrase(&[25.0, -25.0, 25.0]));
    sing(&mut s, &mut lane, breath());
    assert_eq!(lane.phrase_score(), Some(0), "the sloppy second phrase is not averaged with the first");
}

#[test]
fn test_one_note_phrase_has_no_score() {
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0]));
    sing(&mut s, &mut lane, breath());
    assert_eq!(lane.phrase_score(), None);
}

#[test]
fn test_end_phrase_scores_the_unfinished_phrase() {
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0, 0.0]));
    assert!(lane.end_phrase());
    assert_eq!(lane.phrase_score(), Some(100));
    assert!(!lane.end_phrase(), "nothing left to score");
}

#[test]
fn test_view_places_notes_in_time_and_spans_at_least_six_semitones() {
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0, 30.0]));
    let view = lane.view(48);
    assert_eq!(view.bars.len(), 2);
    let (a, b) = (view.bars[0], view.bars[1]);
    assert!(a.x0 < a.x1 && a.x1 <= b.x0 && b.x1 <= 1.0, "left to right in time: {a:?} {b:?}");
    assert!((b.cents - 30.0).abs() < 0.5, "keeps how far off the note was: {}", b.cents);
    assert!(view.high - view.low >= 6 && view.low <= 60 && view.high >= 61, "{view:?}");
}

#[test]
fn test_view_drops_notes_older_than_the_window() {
    let (mut s, mut lane) = (TuningScorer::new(), NoteLane::new());
    sing(&mut s, &mut lane, phrase(&[0.0]));
    sing(&mut s, &mut lane, vec![None; 200]);
    assert_eq!(lane.view(100).bars.len(), 0);
    assert_eq!(lane.view(400).bars.len(), 1);
}

#[test]
fn test_lane_keeps_a_bounded_number_of_notes() {
    let mut lane = NoteLane::new();
    for i in 0..LANE_NOTES + 10 {
        lane.push(true, Some(HeldNote { center_midi: 60.0 + (i % 3) as f32, frames: 8 }));
    }
    assert_eq!(lane.view(u64::MAX).bars.len(), LANE_NOTES);
}

#[test]
fn test_push_reports_change_only_when_something_new_shows() {
    let mut lane = NoteLane::new();
    assert!(!lane.push(true, None));
    assert!(lane.push(true, Some(HeldNote { center_midi: 60.0, frames: 10 })));
    let changes = (0..PHRASE_GAP_FRAMES + 5).filter(|_| lane.push(false, None)).count();
    assert_eq!(changes, 1, "the phrase ends once");
}
