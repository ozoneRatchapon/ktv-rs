use app::booth::{Booth, Placement, Requester};
use app::catalog::builtin_catalog;
use app::room::{
    booth_state, is_valid_key, key_from_bytes, phone_url, refusal, room_of, socket_url, BoothMessage, PhoneCommand,
    RemoteConfig, RoomId, RoomKey, ServerMessage, STATE_MEDLEYS, STATE_NEXT,
};

/// The same vector is checked against `room_of` in worker/protocol.js (tests/room_protocol.test.mjs).
const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const ROOM: &str = "DwBzhbb51LfusnSGBa_hqY";

#[test]
fn test_room_id_matches_the_worker() {
    assert_eq!(room_of(&RoomKey(KEY.to_string())), RoomId(ROOM.to_string()));
}

#[test]
fn test_keys_are_43_base64url_chars() {
    assert_eq!(key_from_bytes(&[0; 32]).0, KEY);
    let key = key_from_bytes(&[0xfb; 32]);
    assert_eq!(key.0.len(), 43);
    assert!(key.0.contains('-') && key.0.contains('_'), "url-safe alphabet: {}", key.0);
    assert!(is_valid_key(&key));
    for bad in ["", "short", &format!("{KEY}A"), &KEY.replace('A', "+")] {
        assert!(!is_valid_key(&RoomKey(bad.to_string())), "{bad}");
    }
}

#[test]
fn test_links() {
    let room = RoomId(ROOM.to_string());
    assert_eq!(phone_url("https://ktv.example", &room), format!("https://ktv.example/remote#r={ROOM}"));
    assert_eq!(socket_url("https://ktv.example", &room).as_deref(), Some(format!("wss://ktv.example/api/room/{ROOM}?role=booth").as_str()));
    assert_eq!(socket_url("http://localhost:8788", &room).as_deref(), Some(format!("ws://localhost:8788/api/room/{ROOM}?role=booth").as_str()));
    assert_eq!(socket_url("file://", &room), None);
}

#[test]
fn test_phone_commands_parse_as_the_worker_forwards_them() {
    let parse = |text: &str| serde_json::from_str::<ServerMessage>(text).ok();
    assert_eq!(
        parse(r#"{"t":"cmd","from":7,"cmd":"queue","code":"10004"}"#),
        Some(ServerMessage::Cmd { from: 7, cmd: PhoneCommand::Queue { code: "10004".into() } })
    );
    assert_eq!(parse(r#"{"t":"cmd","from":1,"cmd":"skip"}"#), Some(ServerMessage::Cmd { from: 1, cmd: PhoneCommand::Skip }));
    assert_eq!(parse(r#"{"t":"cmd","from":1,"cmd":"volume_up"}"#), Some(ServerMessage::Cmd { from: 1, cmd: PhoneCommand::VolumeUp }));
    assert_eq!(parse(r#"{"t":"cmd","from":1,"cmd":"volume_down"}"#), Some(ServerMessage::Cmd { from: 1, cmd: PhoneCommand::VolumeDown }));
    assert_eq!(
        parse(r#"{"t":"cmd","from":2,"cmd":"queue_medley","title":"ยาวๆ"}"#),
        Some(ServerMessage::Cmd { from: 2, cmd: PhoneCommand::QueueMedley { title: "ยาวๆ".into() } })
    );
    assert_eq!(parse(r#"{"t":"phones","n":3}"#), Some(ServerMessage::Phones { n: 3 }));
    assert_eq!(parse(r#"{"t":"cmd","from":1,"cmd":"eject"}"#), None);
    assert_eq!(parse(r#"{"t":"hello"}"#), None);
}

#[test]
fn test_booth_messages_have_the_worker_shape() {
    let json = |msg: &BoothMessage| serde_json::to_value(msg).unwrap();
    assert_eq!(json(&BoothMessage::Hello { key: KEY.into() }), serde_json::json!({ "t": "hello", "key": KEY }));
    assert_eq!(
        json(&BoothMessage::Reply { to: 7, ok: false, text: "No".into() }),
        serde_json::json!({ "t": "reply", "to": 7, "ok": false, "text": "No" })
    );
    assert_eq!(json(&BoothMessage::CloseRoom), serde_json::json!({ "t": "close_room" }));
    let state = json(&BoothMessage::State { state: booth_state("Room 1", None, &[], false, 85) });
    assert_eq!(state["t"], "state");
    assert_eq!(state["state"]["now"], serde_json::Value::Null);
}

#[test]
fn test_state_sends_the_next_few_and_counts_the_rest() {
    let mut booth = Booth::default();
    for song in builtin_catalog().iter().take(STATE_NEXT + 3) {
        booth.add(song.clone(), Requester::Phone, Placement::Back);
    }
    let state = booth_state("Room 1", booth.current.as_ref(), &booth.queue, true, 60);
    let now = state.now.expect("a song on stage");
    assert_eq!(now.code, builtin_catalog()[0].code);
    assert_eq!(now.requester, "📱 Phone");
    assert_eq!(state.next.len(), STATE_NEXT);
    assert_eq!(state.next[0].title, builtin_catalog()[1].title);
    assert_eq!(state.waiting, STATE_NEXT + 2);
    assert!(state.playback);
    assert_eq!(state.volume, 60);
    // State bytes stay far under the Worker's 8 KB booth message cap
    let bytes = serde_json::to_string(&BoothMessage::State { state: booth_state("Room 1", booth.current.as_ref(), &booth.queue, true, 60) }).unwrap().len();
    assert!(bytes < 2048, "{bytes} bytes");
}

#[test]
fn test_phones_queue_only_unless_the_host_allows_playback() {
    let queue_only = RemoteConfig { enabled: true, allow_playback: false };
    let playback = RemoteConfig { enabled: true, allow_playback: true };
    assert_eq!(refusal(&PhoneCommand::Queue { code: "10004".into() }, &queue_only), None);
    assert_eq!(refusal(&PhoneCommand::QueueMedley { title: "Mix".into() }, &queue_only), None, "a medley is queued, not playback");
    for cmd in [PhoneCommand::Skip, PhoneCommand::Pause, PhoneCommand::Replay, PhoneCommand::VolumeUp, PhoneCommand::VolumeDown] {
        assert!(refusal(&cmd, &queue_only).is_some(), "{cmd:?}");
        assert_eq!(refusal(&cmd, &playback), None, "{cmd:?}");
    }
}

#[test]
fn test_remote_config_defaults_off_for_old_settings() {
    let json = serde_json::to_string(&app::types::AppSettings::default()).unwrap();
    let before_remote = json.replace(r#","remote":{"enabled":false,"allow_playback":false}"#, "");
    assert!(!before_remote.contains("remote"), "{before_remote}");
    let old: app::types::AppSettings = serde_json::from_str(&before_remote).unwrap();
    assert_eq!(old.remote, RemoteConfig::default());
}

#[test]
fn test_volume_steps_stay_within_0_to_100() {
    let mut settings = app::types::AppSettings::default();
    assert_eq!(settings.volume(), 85, "default");
    assert_eq!(settings.volume_by(10), 95);
    assert_eq!(settings.volume_by(10), 100, "never above 100");
    assert_eq!(settings.volume_by(-250), 0, "never below 0");
    settings.volume = 400; // a hand-edited save
    assert_eq!(settings.volume(), 100);
    assert_eq!(settings.volume_by(-5), 95);
}

#[test]
fn test_state_lists_saved_medleys_by_title_and_stays_under_the_worker_cap() {
    use app::medley::{MedleyBook, MAX_TITLE_CHARS};
    let empty = serde_json::to_value(booth_state("Room 1", None, &[], false, 85)).unwrap();
    assert!(empty.get("medleys").is_none(), "no saved medleys: nothing extra on the wire");

    let mut book = MedleyBook::default();
    for song in builtin_catalog().iter().take(2) {
        book.add_song(song).unwrap();
    }
    book.save_draft().unwrap();
    let state = booth_state("Room 1", None, &[], false, 85).with_medleys(&book.saved);
    assert_eq!(serde_json::to_value(&state).unwrap()["medleys"], serde_json::json!([{ "title": "Medley 1", "parts": 2 }]));

    // A full book of longest Thai titles, beside a full queue: still far under 8 KB
    let mut booth = Booth::default();
    for song in builtin_catalog().iter().take(STATE_NEXT + 3) {
        booth.add(song.clone(), Requester::Phone, Placement::Back);
    }
    let mut saved = Vec::new();
    for i in 0..STATE_MEDLEYS + 5 {
        let mut medley = book.saved[0].clone();
        medley.title = format!("{i:02}{}", "ก".repeat(MAX_TITLE_CHARS - 2));
        saved.push(medley);
    }
    let state = booth_state("Room 1", booth.current.as_ref(), &booth.queue, true, 60).with_medleys(&saved);
    assert_eq!(state.medleys.len(), STATE_MEDLEYS);
    let text = serde_json::to_string(&BoothMessage::State { state }).unwrap();
    assert!(text.chars().count() < 4096, "{} characters", text.chars().count());
}
