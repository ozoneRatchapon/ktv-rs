use app::timing::{
    apply_overrides, catalog_snippet, fit, mark_at, new_guide, nudge, FitError, GuideOverrides, SyncMark,
};
use app::types::{GuideTrack, Song};

fn guide(offset_secs: f32, rate: f32) -> GuideTrack {
    GuideTrack { video_id: "qdaeYIGnpiA".to_string(), offset_secs, rate }
}

fn song(id: &str, guide: Option<GuideTrack>) -> Song {
    Song {
        id: id.to_string(),
        code: "10001".to_string(),
        title: "t".to_string(),
        artist: "a".to_string(),
        aliases: Vec::new(),
        youtube_id: "inGSjouS77g".to_string(),
        guide,
        duration_secs: 200,
        intro_skip_secs: 18,
        category: "Pop".to_string(),
        channel: "GMM Karaoke".to_string(),
        is_favorite: false,
    }
}

#[test]
fn mark_follows_mapping() {
    let m = mark_at(&guide(-18.0, 1.0), 60.0);
    assert_eq!(m, SyncMark { karaoke_secs: 60.0, mv_secs: 42.0 });
    let m = mark_at(&guide(2.0, 0.5), 10.0);
    assert!((m.mv_secs - 7.0).abs() < 1e-9);
}

#[test]
fn nudge_rounds_to_centiseconds() {
    let g = nudge(&guide(-18.34, 1.0), 0.1);
    assert!((g.offset_secs - -18.24).abs() < 1e-4, "{}", g.offset_secs);
    let g = nudge(&g, -1.0);
    assert!((g.offset_secs - -19.24).abs() < 1e-4);
    assert_eq!(g.video_id, "qdaeYIGnpiA");
    assert_eq!(g.rate, 1.0);
}

#[test]
fn fit_recovers_offset_and_rate() {
    // True mapping: MV = -12.5 + 1.002 * karaoke
    let truth = |k: f64| -12.5 + 1.002 * k;
    let a = SyncMark { karaoke_secs: 40.0, mv_secs: truth(40.0) };
    let b = SyncMark { karaoke_secs: 190.0, mv_secs: truth(190.0) };
    let g = fit("qdaeYIGnpiA", a, b).expect("fit");
    assert!((g.rate - 1.002).abs() < 1e-4, "rate {}", g.rate);
    assert!((g.offset_secs - -12.5).abs() < 0.011, "offset {}", g.offset_secs);
    // Marks in either order give the same line
    assert_eq!(fit("qdaeYIGnpiA", b, a).expect("fit"), g);
}

#[test]
fn fit_after_nudging_at_second_mark() {
    // Curator syncs at 30 s with rate 1, later hears the guide 0.4 s behind at 200 s and nudges +0.4
    let start = guide(-18.0, 1.0);
    let a = mark_at(&start, 30.0);
    let b = mark_at(&nudge(&start, 0.4), 200.0);
    let g = fit(&start.video_id, a, b).expect("fit");
    // The fitted line passes through both marks
    assert!((mark_at(&g, 30.0).mv_secs - a.mv_secs).abs() < 0.02);
    assert!((mark_at(&g, 200.0).mv_secs - b.mv_secs).abs() < 0.02);
    assert!(g.rate > 1.0);
}

#[test]
fn fit_rejects_close_marks() {
    let a = SyncMark { karaoke_secs: 50.0, mv_secs: 32.0 };
    let b = SyncMark { karaoke_secs: 70.0, mv_secs: 52.0 };
    assert!(matches!(fit("x", a, b), Err(FitError::TooClose { .. })));
    assert!(fit("x", a, a).is_err());
}

#[test]
fn fit_rejects_implausible_rate() {
    let a = SyncMark { karaoke_secs: 20.0, mv_secs: 0.0 };
    let b = SyncMark { karaoke_secs: 120.0, mv_secs: 150.0 };
    match fit("x", a, b) {
        Err(FitError::RateOutOfRange { rate }) => assert!((rate - 1.5).abs() < 1e-9),
        other => panic!("{other:?}"),
    }
}

#[test]
fn new_guide_starts_on_same_timeline() {
    assert_eq!(new_guide("dj-iph1Nt0Y"), GuideTrack { video_id: "dj-iph1Nt0Y".to_string(), offset_secs: 0.0, rate: 1.0 });
}

#[test]
fn overrides_replace_only_listed_songs() {
    let mut songs = [song("a", Some(guide(-18.0, 1.0))), song("b", None), song("c", Some(guide(-5.0, 1.0)))];
    let mut overrides = GuideOverrides::new();
    overrides.insert("a".to_string(), guide(-17.5, 1.001));
    overrides.insert("b".to_string(), guide(3.0, 1.0));
    apply_overrides(songs.iter_mut(), &overrides);
    assert_eq!(songs[0].guide, Some(guide(-17.5, 1.001)));
    assert_eq!(songs[1].guide, Some(guide(3.0, 1.0)));
    assert_eq!(songs[2].guide, Some(guide(-5.0, 1.0)));
}

#[test]
fn snippet_is_valid_catalog_json() {
    let g = guide(-18.34, 1.0);
    let json = format!("{{{}}}", catalog_snippet(&g));
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let back: GuideTrack = serde_json::from_value(parsed["guide"].clone()).expect("GuideTrack");
    assert_eq!(back, g);
}
