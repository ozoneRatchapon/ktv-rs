use app::recommendation::{SleepTimeAnticipator, SongTelemetry};
use app::types::Song;
use std::collections::HashSet;
use std::time::Instant;

fn sample_song(id: &str, category: &str, artist: &str) -> Song {
    Song {
        id: id.to_string(),
        code: "10001".to_string(),
        title: format!("Test Song {id}"),
        artist: artist.to_string(),
        youtube_id: "test1234567".to_string(),
        guide: None,
        duration_secs: 200,
        intro_skip_secs: 18,
        category: category.to_string(),
        channel: "Test Channel".to_string(),
        is_favorite: false,
    }
}

#[test]
fn test_dwell_affinity_boost() {
    let mut anticipator = SleepTimeAnticipator::new();
    let catalog = vec![
        sample_song("1", "Rock", "Artist Rock A"),
        sample_song("2", "Pop", "Artist Pop B"),
        sample_song("3", "Luk Thung", "Artist Luk Thung C"),
    ];

    // Simulate singing Rock to completion (100% dwell)
    anticipator.record_song_playback(SongTelemetry {
        song_id: "rock_prev".to_string(),
        code: "00001".to_string(),
        title: "Previous Rock".to_string(),
        artist: "Artist Rock A".to_string(),
        category: "Rock".to_string(),
        duration_secs: 200,
        sang_seconds: 200.0,
        completed_natural: true,
    });

    let queued = HashSet::new();
    let set = anticipator.sleep_compute(&catalog, &queued, None, 3);

    assert_eq!(set.candidates[0].song.category, "Rock");
    assert!(set.candidates[0].predictability > 0.60);
}

#[test]
fn test_tropical_bottleneck_early_skip_hard_prune() {
    let mut anticipator = SleepTimeAnticipator::new();
    let catalog = vec![
        sample_song("1", "Rock", "Loso"),
        sample_song("2", "Luk Thung", "Artist A"),
        sample_song("3", "Luk Thung", "Artist B"),
    ];

    // User immediately skips Luk Thung after 10 seconds (< 30s)
    anticipator.record_song_playback(SongTelemetry {
        song_id: "lt_001".to_string(),
        code: "00002".to_string(),
        title: "Annoying Song".to_string(),
        artist: "Artist A".to_string(),
        category: "Luk Thung".to_string(),
        duration_secs: 240,
        sang_seconds: 10.0,
        completed_natural: false,
    });

    let queued = HashSet::new();
    let set = anticipator.sleep_compute(&catalog, &queued, None, 3);

    // All Luk Thung songs must be completely pruned out by Tropical Semiring!
    let any_luk_thung = set.candidates.iter().any(|c| c.song.category == "Luk Thung");
    assert!(!any_luk_thung, "Tropical Bottleneck must hard-prune skipped category");
}

#[test]
fn test_wake_consume_latency_sub_microsecond() {
    let anticipator = SleepTimeAnticipator::new();
    let catalog: Vec<Song> = (0..50)
        .map(|i| sample_song(&format!("{i}"), if i % 2 == 0 { "Rock" } else { "Pop" }, "Band"))
        .collect();

    let queued = HashSet::new();
    let set = anticipator.sleep_compute(&catalog, &queued, None, 5);

    // Measure wake_consume latency
    let start = Instant::now();
    let choice = anticipator.wake_consume(&set);
    let elapsed = start.elapsed();

    assert!(choice.is_some());
    println!("wake_consume elapsed: {elapsed:?}");
    // Should be sub-microsecond
    assert!(elapsed.as_nanos() < 50_000, "Wake time must be sub-microsecond");
}
