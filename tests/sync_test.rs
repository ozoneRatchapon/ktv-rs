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
fn test_commands_call_core_methods_with_typed_args() {
    use app::js_bridge::JsArg::{Bool, Num};
    let cases = [
        (SyncCommand::LoadSong { offset_secs: -12.5, rate: 1.0 }, "load_song", vec![Num(-12.5), Num(1.0)]),
        (SyncCommand::SetStart(18), "set_start", vec![Num(18.0)]),
        (SyncCommand::TogglePlayback, "toggle_playback", vec![]),
        (SyncCommand::SeekTo(95), "seek_all", vec![Num(95.0)]),
        (SyncCommand::SeekBy(-5), "seek_by", vec![Num(-5.0)]),
        (SyncCommand::Restart(18), "restart", vec![Num(18.0)]),
        (SyncCommand::SwitchVocal { original: true }, "switch_vocal", vec![Bool(true)]),
        (SyncCommand::SetMapping { offset_secs: -18.24, rate: 1.0012 }, "set_mapping", vec![Num(-18.24), Num(1.0012)]),
        (SyncCommand::SetMonitor(true), "set_monitor", vec![Bool(true)]),
    ];
    for (cmd, method, args) in cases {
        assert_eq!(cmd.call(), (method, args), "{cmd:?}");
        assert!(SYNC_JS.contains(&format!("function {method}(")), "ktv_sync.js must define {method} for {cmd:?}");
    }
}

#[test]
fn test_core_is_reachable_without_eval() {
    // Rust calls window.KtvSyncCore.install(window, send) and window.KtvSync.<method>(...) through Reflect
    assert!(SYNC_JS.contains("root.KtvSyncCore = api"));
    assert!(SYNC_JS.contains("win.KtvSync = sync"));
    assert!(SYNC_JS.contains("function install(win, send)"));
    assert!(SYNC_JS.contains("karaoke_time"), "debug() exposes karaoke_time for sync::karaoke_time");
}

#[test]
fn test_frame_ids_match_js_core() {
    assert!(SYNC_JS.contains(&format!("KARAOKE: '{KARAOKE_FRAME_ID}'")));
    assert!(SYNC_JS.contains(&format!("GUIDE: '{GUIDE_FRAME_ID}'")));
}
