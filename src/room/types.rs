//! Phone remote wire types (roadmap Phase 3). JSON with a `t` tag, as `worker/protocol.js` parses it.

use serde::{Deserialize, Serialize};

/// The booth's secret: 43 base64url chars (32 random bytes). Whoever holds it is the room's booth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomKey(pub String);

/// The room's public id: 22 base64url chars from SHA-256 of the key (see [`super::room_of`]). In the phone link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomId(pub String);

/// Booth settings for the remote (saved with the other settings; the key is stored on its own).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Connect to the room and show the phone QR (off by default: no network until the host turns it on).
    #[serde(default)]
    pub enabled: bool,
    /// Phones may also skip, pause and replay. Off: phones can only queue songs (safe in a bar full of guests).
    #[serde(default)]
    pub allow_playback: bool,
}

/// What a phone asked for. Codes are checked (5 digits) by the Worker and again by the booth.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum PhoneCommand {
    Queue { code: String },
    Skip,
    Pause,
    Replay,
}

impl PhoneCommand {
    /// Skip / pause / replay: allowed only when the host turned on [`RemoteConfig::allow_playback`].
    pub fn is_playback(&self) -> bool {
        !matches!(self, Self::Queue { .. })
    }
}

/// Worker → booth.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Phones connected to the room right now.
    Phones { n: u32 },
    Cmd {
        from: u32,
        #[serde(flatten)]
        cmd: PhoneCommand,
    },
}

/// Booth → Worker.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum BoothMessage {
    Hello { key: String },
    State { state: BoothState },
    Reply { to: u32, ok: bool, text: String },
    /// The booth made a new link: phones on this room are disconnected and told to scan again.
    CloseRoom,
}

/// What phones see: the song on stage, the next few, and whether playback buttons work.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct BoothState {
    pub room: String,
    pub now: Option<StateSong>,
    pub next: Vec<StateSong>,
    /// Songs waiting in total (`next` holds only the first few).
    pub waiting: usize,
    pub playback: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StateSong {
    pub code: String,
    pub title: String,
    pub artist: String,
    /// Paid on-chain (★ TIP) or queued from a phone: shown so guests see why a song jumped ahead.
    pub requester: String,
}

/// What the booth's socket reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SocketEvent {
    Open,
    Message(String),
    Closed(u16),
}

/// The booth's link to its room, for the Keypad tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkStatus {
    #[default]
    Off,
    Connecting,
    Online { phones: u32 },
    /// Another tab or device opened this room as the booth; this one stopped (so two tabs never fight).
    Replaced,
}

/// Close codes from `worker/protocol.js` (`CLOSE`).
pub const CLOSE_REPLACED: u16 = 4001;
