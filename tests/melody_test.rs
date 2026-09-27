use app::score::{
    octave_cents, stored_melody, Melody, MelodyScorer, StoredMelodies, TargetNote, TakeResult, TuningSummary, FULL_COVERAGE,
    MIN_TARGET_FRAMES, SING_LAG_SECS,
};

const FRAME: f64 = 0.02;

fn note(start: f32, end: f32, midi: u8) -> TargetNote {
    TargetNote { start, end, midi }
}

/// A4 held for 10 s from t = 1.
fn a4_melody() -> Melody {
    Melody::new([note(1.0, 11.0, 69)]).unwrap()
}

/// Feed `secs` of frames starting at `from`, singing `pitch(t)`.
fn sing(scorer: &mut MelodyScorer, melody: &Melody, from: f64, secs: f64, pitch: impl Fn(f64) -> Option<f32>) {
    let frames = (secs / FRAME).round() as usize;
    for i in 0..frames {
        let t = from + i as f64 * FRAME;
        scorer.push(melody, t, pitch(t));
    }
}

#[test]
fn test_octave_folding() {
    assert_eq!(octave_cents(69.0, 69), 0.0);
    assert_eq!(octave_cents(57.0, 69), 0.0, "an octave down is the same note");
    assert_eq!(octave_cents(81.0, 69), 0.0, "an octave up too");
    assert!((octave_cents(69.3, 69) - 30.0).abs() < 1e-3);
    assert!((octave_cents(56.7, 69) + 30.0).abs() < 1e-3, "flat below, across the octave boundary");
    assert!((octave_cents(70.0, 69) - 100.0).abs() < 1e-3);
    assert!((octave_cents(63.0, 69).abs() - 600.0).abs() < 1e-3, "a tritone is as far as it gets");
}

#[test]
fn test_right_note_in_any_octave_scores_full() {
    let melody = a4_melody();
    for sung in [69.0, 57.0, 45.0, 69.2] {
        let mut scorer = MelodyScorer::new();
        sing(&mut scorer, &melody, 0.0, 12.0, |_| Some(sung));
        assert_eq!(scorer.summary().score(), Some(100), "sung {sung}");
    }
}

#[test]
fn test_wrong_note_and_silence_score_zero() {
    let melody = a4_melody();
    let mut wrong = MelodyScorer::new();
    sing(&mut wrong, &melody, 0.0, 12.0, |_| Some(71.0)); // B4: a whole tone off
    assert_eq!(wrong.summary().score(), Some(0));
    let mut silent = MelodyScorer::new();
    sing(&mut silent, &melody, 0.0, 12.0, |_| None);
    assert_eq!(silent.summary().score(), Some(0));
    assert_eq!(silent.summary().hit_frames, 0);
}

#[test]
fn test_coverage_maps_linearly_up_to_full() {
    let melody = a4_melody();
    let mut half = MelodyScorer::new();
    // Right note in every other second of the tune: 50% coverage
    sing(&mut half, &melody, 0.0, 12.0, |t| ((t - SING_LAG_SECS).floor() as i64 % 2 == 0).then_some(69.0));
    let score = f32::from(half.summary().score().unwrap());
    let expected = 0.5 / FULL_COVERAGE * 100.0;
    assert!((score - expected).abs() <= 3.0, "{score} vs {expected}");
}

#[test]
fn test_only_the_tunes_notes_count_and_a_score_needs_enough_of_them() {
    let melody = Melody::new([note(5.0, 6.0, 60)]).unwrap();
    let mut scorer = MelodyScorer::new();
    sing(&mut scorer, &melody, 0.0, 4.0, |_| Some(62.0)); // before the tune starts: nothing judged
    assert_eq!(scorer.summary().target_frames, 0);
    sing(&mut scorer, &melody, 4.0, 3.0, |_| Some(60.0));
    let frames = scorer.summary().target_frames;
    assert!((45..=55).contains(&frames), "one second of tune: {frames} frames");
    assert!(frames < MIN_TARGET_FRAMES);
    assert_eq!(scorer.summary().score(), None, "one second is not enough to score");
}

#[test]
fn test_paused_video_is_not_judged() {
    let melody = a4_melody();
    let mut scorer = MelodyScorer::new();
    sing(&mut scorer, &melody, 2.0, 1.0, |_| Some(69.0));
    scorer.push(&melody, 3.0, Some(69.0)); // the last frame before the pause is still judged
    let before = scorer.summary();
    for _ in 0..500 {
        assert!(!scorer.push(&melody, 3.0, None), "clock stands still: nothing judged");
    }
    assert_eq!(scorer.summary(), before);
}

#[test]
fn test_late_entry_within_the_edge_window_still_counts() {
    // C4 until 2 s, then D4: a singer still on C4 just after the change hits the edge window
    let melody = Melody::new([note(0.0, 2.0, 60), note(2.0, 4.0, 62)]).unwrap();
    let mut scorer = MelodyScorer::new();
    let t = 2.0 + SING_LAG_SECS + 0.05;
    assert!(scorer.push(&melody, t, Some(60.0)));
    assert_eq!(scorer.summary().hit_frames, 1);
    let mut late = MelodyScorer::new();
    assert!(late.push(&melody, 2.0 + SING_LAG_SECS + 0.3, Some(60.0)));
    assert_eq!(late.summary().hit_frames, 0, "0.3 s late is the wrong note");
}

#[test]
fn test_melody_validation_and_lookup() {
    let melody = Melody::new([
        note(4.0, 5.0, 64),
        note(0.0, 1.0, 60),
        note(2.0, 1.0, 62),        // ends before it starts
        note(f32::NAN, 3.0, 62),   // not a time
        note(3.0, 4.0, 10),        // below the range
        note(6.0, 100.0, 67),      // cut to 30 s
    ])
    .unwrap();
    let starts: Vec<f32> = melody.notes().iter().map(|n| n.start).collect();
    assert_eq!(starts, [0.0, 4.0, 6.0], "sorted, invalid notes dropped");
    assert_eq!(melody.notes()[2].end, 36.0);
    assert_eq!(melody.at(0.5).map(|n| n.midi), Some(60));
    assert_eq!(melody.at(2.5), None);
    assert_eq!(melody.at(35.0).map(|n| n.midi), Some(67), "a long note is found far from its start");
    assert_eq!(melody.between(0.5, 4.5).count(), 2);
    assert!(Melody::new([note(1.0, 0.5, 60)]).is_none(), "nothing valid: no melody");
}

#[test]
fn test_stored_melodies_by_song_id() {
    let store: StoredMelodies = serde_json::from_str(r#"{"cat_1": [[0.5, 1.0, 60], [1.0, 2.0, 62]], "cat_2": [[1, 0, 60]]}"#).unwrap();
    assert_eq!(stored_melody(&store, "cat_1").map(|m| m.notes().len()), Some(2));
    assert!(stored_melody(&store, "cat_2").is_none(), "only invalid notes");
    assert!(stored_melody(&store, "missing").is_none());
}

#[test]
fn test_take_result_keeps_the_melody_score_and_old_saves_still_load() {
    let summary = TuningSummary { notes: 5, mean_abs_cents: Some(10.0) };
    let take = TakeResult::new("cat_1", "t", "a", summary, 1.0).unwrap().with_melody_score(Some(72));
    let json = serde_json::to_string(&take).unwrap();
    assert!(json.contains(r#""melody_score":72"#));
    let without = TakeResult::new("cat_1", "t", "a", summary, 1.0).unwrap();
    let old_json = serde_json::to_string(&without).unwrap();
    assert!(!old_json.contains("melody_score"), "no field when the song had no melody");
    let old: TakeResult = serde_json::from_str(&old_json).unwrap();
    assert_eq!(old.melody_score, None);
}
