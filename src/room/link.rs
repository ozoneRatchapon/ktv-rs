//! Room key → room id, the phone link, the socket URL and the state phones see. Pure, tested on the host.

use sha2::{Digest, Sha256};

use super::types::{BoothState, RoomId, RoomKey, StateSong};
use crate::types::QueueItem;

/// Songs after the current one sent to phones (the rest is only counted).
pub const STATE_NEXT: usize = 5;

/// A key from 32 random bytes.
pub fn key_from_bytes(bytes: &[u8; 32]) -> RoomKey {
    RoomKey(base64url(bytes))
}

/// Whether a stored key has the shape [`key_from_bytes`] makes (a hand-edited or old value is replaced).
pub fn is_valid_key(key: &RoomKey) -> bool {
    key.0.len() == 43 && key.0.bytes().all(is_base64url)
}

/// base64url(SHA-256(key)), first 22 chars. Must match `room_of` in `worker/protocol.js` (tests check both).
pub fn room_of(key: &RoomKey) -> RoomId {
    let digest = Sha256::digest(key.0.as_bytes());
    RoomId(base64url(&digest)[..22].to_string())
}

/// The page a phone opens: the room rides in the fragment, so loading the page never sends it to a server.
pub fn phone_url(origin: &str, room: &RoomId) -> String {
    format!("{origin}/remote#r={}", room.0)
}

/// `wss://host/api/room/<room>?role=booth` (or `ws://` for a local `http://` origin).
pub fn socket_url(origin: &str, room: &RoomId) -> Option<String> {
    let host = origin.strip_prefix("https://").map(|h| ("wss", h)).or_else(|| origin.strip_prefix("http://").map(|h| ("ws", h)));
    let (scheme, host) = host?;
    Some(format!("{scheme}://{host}/api/room/{}?role=booth", room.0))
}

/// What phones see of the booth right now.
pub fn booth_state(room: &str, current: Option<&QueueItem>, queue: &[QueueItem], playback: bool, volume: u32) -> BoothState {
    let song = |item: &QueueItem| StateSong {
        code: item.song.code.clone(),
        title: item.song.title.clone(),
        artist: item.song.artist.clone(),
        requester: item.requester.clone(),
    };
    BoothState {
        room: room.to_string(),
        now: current.map(song),
        next: queue.iter().take(STATE_NEXT).map(song).collect(),
        waiting: queue.len(),
        playback,
        volume,
    }
}

fn is_base64url(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// Unpadded base64url (RFC 4648 §5).
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}
