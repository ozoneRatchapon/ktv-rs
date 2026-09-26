use app::catalog::builtin_catalog;
use app::storage::{decode, Session};
use app::timing::GuideOverrides;
use app::types::{AppSettings, GuideTrack, QueueItem, Song};

fn item(queue_id: u64, song: &Song) -> QueueItem {
    QueueItem { queue_id, song: song.clone(), key_shift: 0, requester: "Test".to_string() }
}

fn custom_song() -> Song {
    Song {
        id: "custom_dQw4w9WgXcQ".to_string(),
        code: "99999".to_string(),
        youtube_id: "dQw4w9WgXcQ".to_string(),
        guide: None,
        ..builtin_catalog()[0].clone()
    }
}

#[test]
fn test_settings_saved_before_timing_tools_still_load() {
    let old = r#"{"default_intro_skip_secs":13,"auto_skip_intro":false,"volume":70,"room_name":"Room 9","sound_fx_enabled":true}"#;
    let settings = decode::<AppSettings>(Some(old)).expect("pre-timing-tools settings must decode");
    assert_eq!(settings.room_name, "Room 9");
    assert!(!settings.show_timing_tools);
    assert!(!settings.seen_shortcuts, "old settings must still show the shortcut help once");
}

#[test]
fn test_guide_overrides_round_trip() {
    let mut overrides = GuideOverrides::new();
    overrides.insert("gmm_001".to_string(), GuideTrack { video_id: "qdaeYIGnpiA".to_string(), offset_secs: -18.2, rate: 1.0 });
    let json = serde_json::to_string(&overrides).unwrap();
    assert_eq!(decode::<GuideOverrides>(Some(&json)), Some(overrides));
}

#[test]
fn test_decode_round_trip_and_garbage() {
    let settings = AppSettings { auto_skip_intro: false, room_name: "Room 9".to_string(), ..AppSettings::default() };
    let json = serde_json::to_string(&settings).unwrap();
    assert_eq!(decode::<AppSettings>(Some(&json)), Some(settings));
    assert_eq!(decode::<AppSettings>(None), None);
    assert_eq!(decode::<AppSettings>(Some("not json")), None);
    // Schema drift (missing field) must fall back, not panic
    assert_eq!(decode::<AppSettings>(Some(r#"{"volume":10}"#)), None);
}

#[test]
fn test_capture_keeps_only_custom_songs_once() {
    let builtin = builtin_catalog();
    let custom = custom_song();
    let mut catalog = builtin.to_vec();
    catalog.push(custom.clone());
    catalog.push(custom.clone()); // same URL added twice
    let session = Session::capture(None, vec![], 7, &catalog, builtin);
    assert_eq!(session.custom_songs, vec![custom]);
    assert_eq!(session.next_queue_id, 7);
}

#[test]
fn test_reconcile_refreshes_catalog_songs_and_keeps_custom() {
    let builtin = builtin_catalog();
    let mut stale = builtin[0].clone();
    if let Some(guide) = stale.guide.as_mut() {
        guide.offset_secs += 5.0; // stored before a re-measure
    }
    let custom = custom_song();
    let session = Session {
        current: Some(item(4, &stale)),
        queue: vec![item(9, &custom)],
        next_queue_id: 3, // behind the stored ids
        custom_songs: vec![custom.clone(), builtin[1].clone()],
    }
    .reconcile(builtin);

    assert_eq!(session.current.unwrap().song, builtin[0]);
    assert_eq!(session.queue[0].song, custom);
    assert_eq!(session.next_queue_id, 10);
    // A custom song that is now in the catalog is not duplicated
    assert_eq!(session.custom_songs, vec![custom]);
}

#[test]
fn test_session_survives_json_round_trip() {
    let builtin = builtin_catalog();
    let session = Session::capture(Some(item(1, &builtin[3])), vec![item(2, &builtin[2])], 3, builtin, builtin);
    let json = serde_json::to_string(&session).unwrap();
    assert_eq!(decode::<Session>(Some(&json)), Some(session));
}

#[test]
fn test_reconcile_renumbers_legacy_duplicate_custom_codes() {
    let builtin = builtin_catalog();
    let first = custom_song();
    let second = Song { id: "custom_9bZkp7q19f0".to_string(), youtube_id: "9bZkp7q19f0".to_string(), ..custom_song() };
    let session = Session {
        current: Some(item(1, &second)),
        queue: vec![item(2, &first)],
        next_queue_id: 3,
        custom_songs: vec![first.clone(), second.clone()], // both "99999" (old builds)
    }
    .reconcile(builtin);

    let codes: Vec<&str> = session.custom_songs.iter().map(|s| s.code.as_str()).collect();
    assert_eq!(codes, vec!["99999", "90001"]);
    assert_eq!(session.current.unwrap().song.code, "90001");
    assert_eq!(session.queue[0].song.code, "99999");
}

#[test]
fn test_only_this_apps_keys_are_cleared() {
    use app::storage::is_app_key;
    for key in ["ktv.settings.v1", "ktv.session.v1", "ktv.picks.v1", "_ktv_seek_lead", "_ktv_cold_lead", "_ktv_pair_lead"] {
        assert!(is_app_key(key), "{key}");
    }
    for key in ["yt-player-quality", "ytidb::LAST_RESULT_ENTRY_KEY", "ktv", "my.ktv.key"] {
        assert!(!is_app_key(key), "{key}");
    }
}
