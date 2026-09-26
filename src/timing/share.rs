//! "Share on GitHub": a pre-filled issue (the repo's guide-timing form) for a timing made in the app.

use super::fit::catalog_snippet;
use crate::types::{GuideTrack, Song};

pub const REPO_URL: &str = "https://github.com/ozoneRatchapon/ktv-rs";

/// RFC 3986 percent-encoding of everything but unreserved characters (UTF-8 bytes for Thai).
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// New-issue link with the guide-timing form's fields filled in (GitHub reads field ids from the query).
pub fn share_url(song: &Song, guide: &GuideTrack) -> String {
    let title = format!("Guide timing: #{} {} - {}", song.code, song.title, song.artist);
    let video = format!("https://www.youtube.com/watch?v={}", song.youtube_id);
    let fields = [
        ("template", "guide_timing.yml".to_string()),
        ("title", title),
        ("song_code", song.code.clone()),
        ("karaoke_video", video),
        ("guide_json", catalog_snippet(guide)),
    ];
    let query: Vec<String> = fields.iter().map(|(k, v)| format!("{k}={}", encode(v))).collect();
    format!("{REPO_URL}/issues/new?{}", query.join("&"))
}
