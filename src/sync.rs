//! Typed bridge to the JS player-sync core (`assets/ktv_sync.js`, a classic script in the static `<head>`).
//! Rust sends [`SyncCommand`]s; the core reports back [`SyncEvent`]s. No `eval` (see [`crate::js_bridge`]).

use futures_channel::mpsc::UnboundedReceiver;

use crate::js_bridge::{self, JsArg};

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
    /// Same song, new guide mapping (timing mode or a saved override); the vocal choice is kept.
    SetMapping { offset_secs: f32, rate: f32 },
    /// Timing mode: keep karaoke audio audible under the guide so misalignment is heard as an echo.
    SetMonitor(bool),
}

/// The installed core (`window.KtvSync`) and its loader (`window.KtvSyncCore`).
const CORE: &str = "KtvSync";
const LOADER: &str = "KtvSyncCore";

impl SyncCommand {
    /// Core method and arguments this command calls.
    pub fn call(self) -> (&'static str, Vec<JsArg>) {
        let num = |n: f64| JsArg::Num(n);
        // Shortest decimal of the f32 (what the timing panel shows), not its widened binary value: -18.24, not -18.2399997
        let exact = |x: f32| JsArg::Num(x.to_string().parse().unwrap_or(f64::from(x)));
        match self {
            Self::LoadSong { offset_secs, rate } => ("load_song", vec![exact(offset_secs), exact(rate)]),
            Self::SetStart(sec) => ("set_start", vec![num(sec as f64)]),
            Self::TogglePlayback => ("toggle_playback", vec![]),
            Self::SeekTo(sec) => ("seek_all", vec![num(sec as f64)]),
            Self::SeekBy(delta) => ("seek_by", vec![num(delta as f64)]),
            Self::Restart(sec) => ("restart", vec![num(sec as f64)]),
            Self::SwitchVocal { original } => ("switch_vocal", vec![JsArg::Bool(original)]),
            Self::SetMapping { offset_secs, rate } => ("set_mapping", vec![exact(offset_secs), exact(rate)]),
            Self::SetMonitor(both) => ("set_monitor", vec![JsArg::Bool(both)]),
        }
    }

    /// No-op until the core is installed.
    pub fn run(self) {
        let (method, args) = self.call();
        let _ = js_bridge::call(CORE, method, &args);
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

/// Karaoke playback position with sub-second precision (`Time` events are whole seconds).
/// `None` before the sync core is installed.
pub fn karaoke_time() -> Option<f64> {
    js_bridge::call(CORE, "debug", &[])?.f64("karaoke_time")
}

/// Wire the sync core (once per page; a remount only rebinds the channel) and return its event messages.
/// Call during render, before any effect issues a [`SyncCommand`].
pub fn install() -> UnboundedReceiver<String> {
    js_bridge::install(LOADER)
}
