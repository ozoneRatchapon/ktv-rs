use app::sync::{SyncCommand, SyncEvent, GUIDE_FRAME_ID, KARAOKE_FRAME_ID, SYNC_JS};

#[test]
fn test_parse_sync_events() {
    assert_eq!(SyncEvent::parse("ended"), Some(SyncEvent::Ended));
    assert_eq!(SyncEvent::parse("TIME:83"), Some(SyncEvent::Time(83)));
    assert_eq!(SyncEvent::parse("PAUSE_STATE:1"), Some(SyncEvent::Paused(true)));
    assert_eq!(SyncEvent::parse("PAUSE_STATE:0"), Some(SyncEvent::Paused(false)));
    assert_eq!(SyncEvent::parse("GUIDE_ERROR:150"), Some(SyncEvent::GuideError(150)));
}

#[test]
fn test_parse_rejects_malformed_events() {
    for msg in ["", "TIME:", "TIME:-1", "TIME:1.5", "PAUSE_STATE:yes", "GUIDE_ERROR:x", "UNKNOWN:1", "Ended"] {
        assert_eq!(SyncEvent::parse(msg), None, "{msg:?} must not parse");
    }
}

#[test]
fn test_commands_call_guarded_core_methods() {
    let cases = [
        (SyncCommand::LoadSong { offset_secs: -12.5, rate: 1.0 }, "load_song(-12.5, 1)"),
        (SyncCommand::SetStart(18), "set_start(18)"),
        (SyncCommand::TogglePlayback, "toggle_playback()"),
        (SyncCommand::SeekTo(95), "seek_all(95)"),
        (SyncCommand::SeekBy(-5), "seek_by(-5)"),
        (SyncCommand::Restart(18), "restart(18)"),
        (SyncCommand::SwitchVocal { original: true }, "switch_vocal(true)"),
        (SyncCommand::SetMapping { offset_secs: -18.24, rate: 1.0012 }, "set_mapping(-18.24, 1.0012)"),
        (SyncCommand::SetMonitor(true), "set_monitor(true)"),
    ];
    for (cmd, call) in cases {
        assert_eq!(cmd.to_js(), format!("if (window.KtvSync) {{ window.KtvSync.{call}; }}"));
        let method = call.split('(').next().unwrap_or_default();
        assert!(
            SYNC_JS.contains(&format!("function {method}(")),
            "ktv_sync.js must define {method} for {cmd:?}"
        );
    }
}

#[test]
fn test_frame_ids_match_js_core() {
    assert!(SYNC_JS.contains(&format!("KARAOKE: '{KARAOKE_FRAME_ID}'")));
    assert!(SYNC_JS.contains(&format!("GUIDE: '{GUIDE_FRAME_ID}'")));
}
