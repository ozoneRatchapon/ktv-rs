/// YouTube video IDs are exactly 11 characters from the URL-safe base64 alphabet.
pub const VIDEO_ID_LEN: usize = 11;

fn is_id_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

pub fn is_video_id(s: &str) -> bool {
    s.len() == VIDEO_ID_LEN && s.bytes().all(is_id_byte)
}

/// Markers after which a video ID starts: `watch?v=`, `youtu.be/`, `/shorts/`, `/embed/`, `/live/`, `/v/`.
const ID_MARKERS: [&str; 7] = ["?v=", "&v=", "youtu.be/", "/shorts/", "/embed/", "/live/", "/v/"];

/// Extract a video ID from a bare ID or any common YouTube URL form.
/// Returns `None` unless the result is a well-formed 11-character ID, so junk never reaches the player.
pub fn parse_video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if is_video_id(trimmed) {
        return Some(trimmed.to_string());
    }
    ID_MARKERS.iter().find_map(|marker| {
        let start = trimmed.find(marker)? + marker.len();
        let rest = &trimmed[start..];
        let end = rest.bytes().position(|b| !is_id_byte(b)).unwrap_or(rest.len());
        let id = &rest[..end];
        is_video_id(id).then(|| id.to_string())
    })
}
