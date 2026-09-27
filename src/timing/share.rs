//! "Share on GitHub": a pre-filled issue (the repo's guide-timing form) for a timing made in the app.

use super::fit::catalog_snippet;
use crate::links::percent_encode as encode;
use crate::types::{GuideTrack, Song};

pub const REPO_URL: &str = "https://github.com/ozoneRatchapon/ktv-rs";

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
