//! Phone remote (roadmap Phase 3): the booth joins a room on the site's Worker (one Durable Object per room);
//! guests' phones open `/remote` from a QR, queue songs and, if the host allows, skip / pause / replay.

mod link;
#[cfg(target_arch = "wasm32")]
mod socket;
mod types;
#[cfg(not(target_arch = "wasm32"))]
mod unsupported;

pub use link::{booth_state, is_valid_key, key_from_bytes, phone_url, room_of, socket_url, STATE_NEXT};
#[cfg(target_arch = "wasm32")]
pub use socket::Socket;
pub use types::{
    BoothMessage, BoothState, LinkStatus, PhoneCommand, RemoteConfig, RoomId, RoomKey, ServerMessage, SocketEvent,
    StateSong, CLOSE_REPLACED,
};
#[cfg(not(target_arch = "wasm32"))]
pub use unsupported::Socket;

/// Why the booth refuses a phone's command, or `None` to carry it out.
pub fn refusal(cmd: &PhoneCommand, config: &RemoteConfig) -> Option<&'static str> {
    match (cmd.is_playback(), config.allow_playback) {
        (true, false) => Some("The host lets phones queue songs only"),
        _ => None,
    }
}
