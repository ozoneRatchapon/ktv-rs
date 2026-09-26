//! Booth keyboard bridge: `assets/ktv_keys.js` turns keydowns into messages, [`KeyAction::parse`] types them.

use dioxus::prelude::*;

pub const KEYS_JS: &str = include_str!("../assets/ktv_keys.js");

/// What a key press asks the app to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Append to the search.
    Type(char),
    /// Delete the last search character.
    Backspace,
    /// Clear the search.
    ClearSearch,
    TogglePlayback,
    /// Seek by this many seconds.
    SeekBy(i64),
    ToggleHelp,
}

impl KeyAction {
    pub fn parse(msg: &str) -> Option<Self> {
        match msg {
            "ESC" => return Some(Self::ClearSearch),
            "HELP" => return Some(Self::ToggleHelp),
            "SPACE" => return Some(Self::TogglePlayback),
            "BACKSPACE" => return Some(Self::Backspace),
            _ => {}
        }
        let (tag, value) = msg.split_once(':')?;
        match tag {
            "CHAR" => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Some(Self::Type(c)),
                    _ => None,
                }
            }
            "SEEK_REL" => value.parse().ok().map(Self::SeekBy),
            _ => None,
        }
    }
}

/// Load the key listener (once per page) and bind its messages to the returned eval.
pub fn install() -> document::Eval {
    document::eval(&format!("{KEYS_JS}\nwindow.KtvKeysCore.install(window, (msg) => dioxus.send(msg));"))
}
