//! Booth keyboard and game-controller bridge: `assets/ktv_keys.js` (a classic script in the static `<head>`) turns
//! keydowns and gamepad presses into messages, [`KeyAction::parse`] types them.

use futures_channel::mpsc::UnboundedReceiver;

use crate::js_bridge;

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
    /// Enter: queue the song whose 5-digit code is the whole search.
    Submit,
    /// Media "next track" key or gamepad Start.
    NextSong,
    /// ↑ / ↓ or D-pad up / down: change the volume by this many points (0-100 scale).
    VolumeBy(i32),
}

impl KeyAction {
    pub fn parse(msg: &str) -> Option<Self> {
        match msg {
            "ESC" => return Some(Self::ClearSearch),
            "HELP" => return Some(Self::ToggleHelp),
            "SPACE" => return Some(Self::TogglePlayback),
            "BACKSPACE" => return Some(Self::Backspace),
            "ENTER" => return Some(Self::Submit),
            "NEXT" => return Some(Self::NextSong),
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
            "VOLUME_REL" => value.parse().ok().map(Self::VolumeBy),
            _ => None,
        }
    }
}

/// Wire the key listener (once per page; a remount only rebinds the channel) and return its messages.
pub fn install() -> UnboundedReceiver<String> {
    js_bridge::install("KtvKeysCore")
}
