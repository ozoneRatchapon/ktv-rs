use app::tempo::{add_tap, fit, sanitize, BeatGrid, BeatGrids, COUNT_IN_BEATS, MIN_TAPS, PRE_ROLL_SECS, TAP_GAP_SECS};

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn test_steady_taps_give_tempo_and_phase() {
    // 120 BPM, beats at 10.25 + k/2, tapped a little early and late
    let taps = [10.27, 10.73, 11.26, 11.74, 12.25, 12.77];
    let grid = fit(&taps).unwrap();
    assert!(close(grid.bpm(), 120.0, 1.5), "{}", grid.bpm());
    assert!(close(grid.nearest_beat(20.2), 20.25, 0.03), "{grid:?}");
}

#[test]
fn test_a_missed_beat_and_a_slip_do_not_bend_the_tempo() {
    // 100 BPM (0.6 s): the 4th beat is skipped, and one tap lands 0.25 beat off
    let taps = [5.0, 5.6, 6.2, 7.4, 8.0, 8.75, 9.2];
    let grid = fit(&taps).unwrap();
    assert!(close(grid.bpm(), 100.0, 1.0), "{}", grid.bpm());
}

#[test]
fn test_too_few_taps_or_no_tempo_give_nothing() {
    assert!(fit(&[1.0, 1.5, 2.0]).is_none(), "needs {MIN_TAPS} taps");
    assert!(fit(&[1.0, 1.01, 1.02, 1.03]).is_none(), "faster than any song");
    assert!(fit(&[1.0, 3.0, 5.0, 7.0]).is_none(), "30 BPM is slower than any song");
    assert!(fit(&[2.0, 2.0, 2.0, 2.0]).is_none());
}

#[test]
fn test_count_in_lands_on_the_beat_nearest_the_part() {
    let grid = BeatGrid { period: 0.5, anchor: 0.25 };
    let count = grid.count_in(30.1); // A pressed a little late: the part starts on the 30.25 beat
    assert_eq!(count.land, 30.25);
    assert_eq!(count.clicks, vec![28.25, 28.75, 29.25, 29.75]);
    assert_eq!(count.clicks.len(), COUNT_IN_BEATS as usize);
    assert!(close(count.start, 28.25 - PRE_ROLL_SECS, 1e-9));
}

#[test]
fn test_count_in_at_the_very_start_is_shorter() {
    let grid = BeatGrid { period: 0.5, anchor: 0.0 };
    let count = grid.count_in(1.0);
    assert_eq!(count.clicks, vec![0.0, 0.5], "no clicks before the song starts");
    assert_eq!(count.start, 0.0);
}

#[test]
fn test_a_pause_or_seek_back_starts_a_new_series() {
    let mut taps = Vec::new();
    for t in [1.0, 1.5, 2.0] {
        add_tap(&mut taps, t);
    }
    add_tap(&mut taps, 2.0 + TAP_GAP_SECS + 0.1);
    assert_eq!(taps.len(), 1, "a long pause starts over");
    add_tap(&mut taps, 1.0);
    assert_eq!(taps, vec![1.0], "tapping after a seek back starts over");
}

#[test]
fn test_saved_grids_drop_anything_unusable() {
    let grids: BeatGrids = serde_json::from_str(
        r#"{"a": {"period": 0.5, "anchor": 1.0}, "b": {"period": 0.0, "anchor": 1.0}, "c": {"period": 5.0, "anchor": 0.0}}"#,
    )
    .unwrap();
    let kept = sanitize(grids);
    assert_eq!(kept.len(), 1);
    assert!(kept.contains_key("a"));
}
