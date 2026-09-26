use app::score::{HeldNote, TuningScorer, TuningSummary};

/// Analysis frames per second at 48 kHz with a 1024-sample hop.
const FPS: f32 = 48_000.0 / 1024.0;

fn feed(scorer: &mut TuningScorer, frames: impl IntoIterator<Item = Option<f32>>) -> Vec<HeldNote> {
    let mut notes: Vec<HeldNote> = frames.into_iter().filter_map(|f| scorer.push(f)).collect();
    notes.extend(scorer.finish());
    notes
}

fn held(midi: f32, frames: usize) -> Vec<Option<f32>> {
    vec![Some(midi); frames]
}

fn silence(frames: usize) -> Vec<Option<f32>> {
    vec![None; frames]
}

/// Melody of held notes, each `cents` off its semitone, separated by short rests.
fn melody(cents: &[f32]) -> Vec<Option<f32>> {
    cents
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            let mut f = held(60.0 + (i % 5) as f32 + c / 100.0, 20);
            f.extend(silence(5));
            f
        })
        .collect()
}

#[test]
fn test_in_tune_notes_score_100() {
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, melody(&[0.0, 2.0, -3.0, 1.0]));
    assert_eq!(notes.len(), 4);
    assert_eq!(s.summary().score(), Some(100));
}

#[test]
fn test_score_falls_with_tuning_error() {
    let score_for = |c: f32| {
        let mut s = TuningScorer::new();
        feed(&mut s, melody(&[c, -c, c, -c]));
        s.summary().score().unwrap()
    };
    let (good, fair, chance) = (score_for(10.0), score_for(18.0), score_for(25.0));
    assert!(good > fair && fair > chance, "{good} {fair} {chance}");
    assert_eq!(chance, 0);
    assert_eq!(score_for(40.0), 0);
}

#[test]
fn test_no_score_before_three_notes() {
    let mut s = TuningScorer::new();
    feed(&mut s, melody(&[0.0, 0.0]));
    assert_eq!(s.summary().notes, 2);
    assert_eq!(s.summary().score(), None);
    assert_eq!(TuningSummary::default().score(), None);
}

#[test]
fn test_vibrato_is_one_in_tune_note() {
    // 6 Hz vibrato, ±40 cents, around an exact D4 for 2 s
    let frames: Vec<Option<f32>> = (0..(2.0 * FPS) as usize)
        .map(|i| Some(62.0 + 0.4 * (std::f32::consts::TAU * 6.0 * i as f32 / FPS).sin()))
        .collect();
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, frames);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].cents_off().abs() < 3.0, "{notes:?}");
}

#[test]
fn test_short_blips_are_not_judged() {
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, [held(60.3, 7), silence(4), held(65.4, 5)].concat());
    assert!(notes.is_empty());
    assert_eq!(s.summary().mean_abs_cents, None);
}

#[test]
fn test_short_dropouts_do_not_split_a_note() {
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, [held(60.0, 10), silence(2), held(60.0, 10)].concat());
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].frames, 20);

    let notes = feed(&mut s, [held(60.0, 10), silence(3), held(60.0, 10)].concat());
    assert_eq!(notes.len(), 2);
}

#[test]
fn test_semitone_step_splits_notes() {
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, [held(60.0, 12), held(61.0, 12)].concat());
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].center_midi, 60.0);
    assert_eq!(notes[1].center_midi, 61.0);
}

#[test]
fn test_slow_glide_is_not_one_note_centered_between_semitones() {
    // Portamento from C4 to D4 over 1 s: a single averaged "note" would land near C#4 and look in tune
    let n = FPS as usize;
    let glide: Vec<Option<f32>> = (0..n).map(|i| Some(60.0 + 2.0 * i as f32 / n as f32)).collect();
    let mut s = TuningScorer::new();
    let notes = feed(&mut s, glide);
    assert!(notes.len() >= 2, "{notes:?}");
    assert!(notes.iter().all(|note| (note.frames as usize) < n));
}

#[test]
fn test_pitches_unrelated_to_semitones_score_near_zero() {
    // Deterministic pseudo-random offsets, uniform over -50..50 cents
    let mut x: u32 = 12345;
    let offsets: Vec<f32> = (0..200)
        .map(|_| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (x >> 8) as f32 / (1u32 << 24) as f32 * 100.0 - 50.0
        })
        .collect();
    let mut s = TuningScorer::new();
    feed(&mut s, melody(&offsets));
    assert_eq!(s.summary().notes, 200);
    assert!(s.summary().score().unwrap() <= 15, "{:?}", s.summary());
}

fn take(title: &str, notes: u32, cents: f32) -> app::score::TakeResult {
    let summary = TuningSummary { notes, mean_abs_cents: Some(cents) };
    app::score::TakeResult::new("id", title, "artist", summary).expect("judged take")
}

#[test]
fn test_take_result_needs_a_judged_note() {
    assert_eq!(app::score::TakeResult::new("id", "t", "a", TuningSummary::default()), None, "mic on but silent");
    let result = take("t", 5, 8.0);
    assert_eq!(result.score(), Some(100));
    assert_eq!(take("t", 2, 8.0).score(), None, "too few notes for a score, still kept");
}

#[test]
fn test_history_is_newest_first_and_capped() {
    let mut history = Vec::new();
    for i in 0..app::score::MAX_RESULTS + 5 {
        app::score::record(&mut history, take(&format!("song {i}"), 4, 10.0));
    }
    assert_eq!(history.len(), app::score::MAX_RESULTS);
    assert_eq!(history[0].title, format!("song {}", app::score::MAX_RESULTS + 4));
}

#[test]
fn test_history_round_trips_through_storage() {
    let history = vec![take("รักไม่ไหวแล้วโว้ย", 12, 11.5)];
    let json = serde_json::to_string(&history).unwrap();
    assert_eq!(app::storage::decode::<Vec<app::score::TakeResult>>(Some(&json)), Some(history));
}
