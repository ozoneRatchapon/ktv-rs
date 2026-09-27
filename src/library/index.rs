//! The song index behind the phone request page's search (`/songs.txt`, built by `examples/songs_index.rs`).

use std::fmt::Write;

use crate::types::Song;

/// One line per song: `code \t title \t artist \t aliases` (aliases space-joined). Tabs and line breaks inside a
/// field become spaces, so every line has exactly four fields. Parsed by `parse_songs` in `public/request.js`.
pub fn song_index<'a>(songs: impl IntoIterator<Item = &'a Song>) -> String {
    let clean = |text: &str| text.replace(['\t', '\n', '\r'], " ");
    let mut out = String::new();
    for song in songs {
        let (title, artist, aliases) = (clean(&song.title), clean(&song.artist), clean(&song.aliases.join(" ")));
        let _ = writeln!(out, "{}\t{title}\t{artist}\t{aliases}", song.code);
    }
    out
}
