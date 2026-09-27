//! Plan 003 A3: the template MC.

use app::mc::{mc_line, seed_of, McEvent, McVoice};
use app::storage::decode;
use app::types::AppSettings;

fn song_up(tipped: bool) -> McEvent {
    McEvent::SongUp { title: "รักไม่ไหวแล้วโว้ย".to_string(), artist: "โจอี้ ภูวศิษฐ์".to_string(), tipped }
}

#[test]
fn off_says_nothing() {
    assert_eq!(mc_line(&song_up(false), McVoice::Off, 0), None);
    assert_eq!(McVoice::Off.lang(), None);
}

#[test]
fn a_song_is_announced_with_title_and_artist_in_either_language() {
    for seed in 0..6 {
        for voice in [McVoice::Thai, McVoice::English] {
            let line = mc_line(&song_up(false), voice, seed).expect("a line");
            assert!(line.contains("รักไม่ไหวแล้วโว้ย") && line.contains("โจอี้ ภูวศิษฐ์"), "{line}");
            assert!(!line.contains('{'), "no template left: {line}");
        }
    }
    assert_eq!(McVoice::Thai.lang(), Some("th-TH"));
    assert_eq!(McVoice::English.lang(), Some("en-US"));
}

#[test]
fn a_tipped_request_gets_its_own_line() {
    let line = mc_line(&song_up(true), McVoice::English, 0).expect("a line");
    assert!(line.starts_with("A tipped request!"), "{line}");
    assert!(mc_line(&song_up(true), McVoice::Thai, 0).expect("a line").contains("ทิป"));
}

#[test]
fn the_same_song_gets_the_same_line_and_songs_vary() {
    let seed = seed_of("gmm-10004");
    assert_eq!(seed, seed_of("gmm-10004"), "stable across calls");
    assert_eq!(mc_line(&song_up(false), McVoice::Thai, seed), mc_line(&song_up(false), McVoice::Thai, seed));
    let lines: std::collections::HashSet<String> =
        (0..3).filter_map(|seed| mc_line(&song_up(false), McVoice::English, seed)).collect();
    assert_eq!(lines.len(), 3, "three wordings");
}

#[test]
fn take_results_follow_the_score_card_bands_and_name_the_singer() {
    let take = |score, singer: Option<&str>| McEvent::TakeEnded { score, singer: singer.map(str::to_string) };
    assert_eq!(mc_line(&take(95, None), McVoice::English, 0).as_deref(), Some("Spot on! 95 points!"));
    assert_eq!(mc_line(&take(75, None), McVoice::Thai, 0).as_deref(), Some("ร้องได้ดีเลย 75 คะแนน"));
    assert_eq!(mc_line(&take(50, Some("Nok")), McVoice::English, 0).as_deref(), Some("Nok, getting there: 50 points. One more?"));
    assert_eq!(mc_line(&take(10, None), McVoice::Thai, 0).as_deref(), Some("ฝึกอีกนิด สู้ๆ"));
}

#[test]
fn the_voice_cycles_and_old_settings_load_with_it_off() {
    assert_eq!(McVoice::Off.next(), McVoice::Thai);
    assert_eq!(McVoice::Thai.next(), McVoice::English);
    assert_eq!(McVoice::English.next(), McVoice::Off);
    let old = r#"{"default_intro_skip_secs":18,"auto_skip_intro":true,"volume":85,"room_name":"A","sound_fx_enabled":true}"#;
    let settings: AppSettings = decode(Some(old)).expect("old settings still load");
    assert_eq!(settings.mc_voice, McVoice::Off);
}
