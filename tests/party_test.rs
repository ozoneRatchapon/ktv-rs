use app::score::{
    clean_name, leaderboard, name_take, recent_singers, TakeResult, TuningSummary, MAX_NAME_CHARS, PARTY_WINDOW_MS,
};

const HOUR_MS: f64 = 3_600_000.0;
const NOW: f64 = 1_790_000_000_000.0;

fn take(singer: Option<&str>, cents: f32, notes: u32, sung_at_ms: f64) -> TakeResult {
    let summary = TuningSummary { notes, mean_abs_cents: Some(cents) };
    let mut t = TakeResult::new("id", "song", "artist", summary, sung_at_ms).expect("judged take");
    t.singer = singer.map(str::to_string);
    t
}

#[test]
fn test_clean_name() {
    assert_eq!(clean_name("  Nok   Noi "), Some("Nok Noi".to_string()));
    assert_eq!(clean_name("ต้น"), Some("ต้น".to_string()));
    assert_eq!(clean_name("   "), None);
    let long = "ก".repeat(MAX_NAME_CHARS + 10);
    assert_eq!(clean_name(&long).unwrap().chars().count(), MAX_NAME_CHARS, "capped by characters, not bytes");
    assert_eq!(clean_name(&format!("{} x", "a".repeat(MAX_NAME_CHARS - 1))), Some("a".repeat(MAX_NAME_CHARS - 1)), "no trailing space after the cap");
}

#[test]
fn test_best_take_per_singer_highest_first() {
    let history = vec![
        take(Some("Nok"), 20.0, 10, NOW - HOUR_MS),
        take(Some("ton"), 9.0, 10, NOW - 2.0 * HOUR_MS),
        take(Some("Nok"), 10.0, 10, NOW - 3.0 * HOUR_MS),
        take(None, 5.0, 10, NOW - HOUR_MS),
        take(Some("Ton"), 15.0, 10, NOW - 4.0 * HOUR_MS),
    ];
    let rows = leaderboard(&history, NOW);
    let summary: Vec<_> = rows.iter().map(|r| (r.singer.as_str(), r.best.score(), r.takes)).collect();
    assert_eq!(summary.len(), 2, "unnamed takes are not ranked; Ton and ton are one singer");
    assert_eq!(summary[0].0, "ton");
    assert_eq!(summary[0].2, 2);
    assert_eq!(summary[1], ("Nok", history[2].score(), 2), "Nok's better take counts");
    assert!(summary[0].1 > summary[1].1);
}

#[test]
fn test_only_tonight_and_only_scored_takes() {
    let history = vec![
        take(Some("Nok"), 9.0, 10, NOW - PARTY_WINDOW_MS - 1.0),
        take(Some("Ton"), 9.0, 2, NOW - HOUR_MS),
        take(Some("Pim"), 12.0, 10, NOW),
    ];
    let rows = leaderboard(&history, NOW);
    assert_eq!(rows.iter().map(|r| r.singer.as_str()).collect::<Vec<_>>(), ["Pim"], "yesterday and unscored takes left out");
}

#[test]
fn test_ties_go_to_more_notes_then_the_earlier_take() {
    let history = vec![
        take(Some("Late"), 8.0, 10, NOW),
        take(Some("Early"), 8.0, 10, NOW - HOUR_MS),
        take(Some("More"), 8.0, 30, NOW),
    ];
    let order: Vec<_> = leaderboard(&history, NOW).into_iter().map(|r| r.singer).collect();
    assert_eq!(order, ["More", "Early", "Late"]);
}

#[test]
fn test_naming_a_take_and_recent_names() {
    let mut history = vec![take(None, 9.0, 10, NOW), take(Some("Nok"), 9.0, 10, NOW - 1.0), take(Some("nok"), 9.0, 10, NOW - 2.0)];
    assert!(name_take(&mut history, NOW, None, Some("Pim".to_string())));
    assert!(!name_take(&mut history, NOW + 5.0, None, Some("x".to_string())), "take no longer kept");
    assert_eq!(recent_singers(&history, 6), ["Pim", "Nok"]);
    assert_eq!(recent_singers(&history, 1), ["Pim"]);
    assert!(name_take(&mut history, NOW, None, None));
    assert_eq!(history[0].singer, None);
}

#[test]
fn test_duet_parts_of_one_take_are_named_apart() {
    let summary = TuningSummary { notes: 5, mean_abs_cents: Some(10.0) };
    let take = |part| TakeResult::new("cat_1", "t", "a", summary, NOW).unwrap().with_part(Some(part));
    let mut history = vec![take(2), take(1)];
    assert!(name_take(&mut history, NOW, Some(1), Some("Pim".to_string())));
    assert!(name_take(&mut history, NOW, Some(2), Some("Ton".to_string())));
    assert_eq!(history[0].singer.as_deref(), Some("Ton"));
    assert_eq!(history[1].singer.as_deref(), Some("Pim"));
    assert!(!name_take(&mut history, NOW, None, Some("x".to_string())), "a duet take has a part");
    assert_eq!(leaderboard(&history, NOW).len(), 2, "both singers rank");
}
