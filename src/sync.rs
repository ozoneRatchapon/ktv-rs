//! Typed bridge to the JS player-sync core (`assets/ktv_sync.js`).
//! Rust sends [`SyncCommand`]s; the core reports back [`SyncEvent`]s over the `dioxus.send` channel.

use dioxus::prelude::*;

pub const SYNC_JS: &str = include_str!("../assets/ktv_sync.js");

/// Iframe ids shared with `FRAME` in `ktv_sync.js`.
pub const KARAOKE_FRAME_ID: &str = "ktv-youtube-player";
pub const GUIDE_FRAME_ID: &str = "ktv-guide-player";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncCommand {
    /// New song: apply the measured guide mapping (MV time = offset + rate * karaoke time).
    LoadSong { offset_secs: f32, rate: f32 },
    /// Karaoke iframe (re)mounted at this second.
    SetStart(u64),
    TogglePlayback,
    SeekTo(u64),
    SeekBy(i64),
    /// Replay: seek both players to this second and resume if paused.
    Restart(u64),
    SwitchVocal { original: bool },
}

impl SyncCommand {
    pub fn to_js(self) -> String {
        let call = match self {
            Self::LoadSong { offset_secs, rate } => format!("load_song({offset_secs}, {rate})"),
            Self::SetStart(sec) => format!("set_start({sec})"),
            Self::TogglePlayback => "toggle_playback()".to_string(),
            Self::SeekTo(sec) => format!("seek_all({sec})"),
            Self::SeekBy(delta) => format!("seek_by({delta})"),
            Self::Restart(sec) => format!("restart({sec})"),
            Self::SwitchVocal { original } => format!("switch_vocal({original})"),
        };
        format!("if (window.KtvSync) {{ window.KtvSync.{call}; }}")
    }

    pub fn run(self) {
        let _ = document::eval(&self.to_js());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncEvent {
    Ended,
    Time(u64),
    Paused(bool),
    /// YouTube player error code from the guide iframe (e.g. 100 removed, 101/150 embedding disabled).
    GuideError(i32),
}

impl SyncEvent {
    pub fn parse(msg: &str) -> Option<Self> {
        if msg == "ended" {
            return Some(Self::Ended);
        }
        let (tag, value) = msg.split_once(':')?;
        match tag {
            "TIME" => value.parse().ok().map(Self::Time),
            "PAUSE_STATE" => match value {
                "1" => Some(Self::Paused(true)),
                "0" => Some(Self::Paused(false)),
                _ => None,
            },
            "GUIDE_ERROR" => value.parse().ok().map(Self::GuideError),
            _ => None,
        }
    }
}

/// Load the sync core (once per page) and bind its event channel to the returned eval.
/// Call during render, before any effect issues a [`SyncCommand`].
pub fn install() -> document::Eval {
    document::eval(&format!(
        "{SYNC_JS}\nwindow.KtvSyncCore.install(window, (msg) => dioxus.send(msg));"
    ))
}
