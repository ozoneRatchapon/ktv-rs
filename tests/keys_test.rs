use app::keys::{KeyAction, KEYS_JS};

#[test]
fn test_parse_every_message() {
    assert_eq!(KeyAction::parse("ESC"), Some(KeyAction::ClearSearch));
    assert_eq!(KeyAction::parse("HELP"), Some(KeyAction::ToggleHelp));
    assert_eq!(KeyAction::parse("SPACE"), Some(KeyAction::TogglePlayback));
    assert_eq!(KeyAction::parse("BACKSPACE"), Some(KeyAction::Backspace));
    assert_eq!(KeyAction::parse("SEEK_REL:-5"), Some(KeyAction::SeekBy(-5)));
    assert_eq!(KeyAction::parse("SEEK_REL:5"), Some(KeyAction::SeekBy(5)));
    assert_eq!(KeyAction::parse("CHAR:a"), Some(KeyAction::Type('a')));
    assert_eq!(KeyAction::parse("CHAR:ก"), Some(KeyAction::Type('ก')), "Thai keyboard input");
    assert_eq!(KeyAction::parse("CHAR::"), Some(KeyAction::Type(':')));
}

#[test]
fn test_parse_rejects_garbage() {
    for msg in ["", "CHAR:", "CHAR:ab", "SEEK_REL:x", "SEEK_REL:", "NOPE:1", "esc", "ended"] {
        assert_eq!(KeyAction::parse(msg), None, "{msg:?}");
    }
}

#[test]
fn test_js_sends_only_messages_rust_parses() {
    // Contract with assets/ktv_keys.js: every message literal it emits has a parser arm here
    for literal in ["'ESC'", "'HELP'", "'SPACE'", "'BACKSPACE'", "'SEEK_REL:'", "'CHAR:'"] {
        assert!(KEYS_JS.contains(literal), "ktv_keys.js no longer sends {literal}");
    }
    assert!(KEYS_JS.contains("root.KtvKeysCore = api"), "install() entry point used by keys::install");
}
