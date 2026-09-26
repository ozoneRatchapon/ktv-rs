use std::sync::OnceLock;

use super::types::{Library, LibraryFile, LibraryRow, Songbook};
use crate::types::Song;

/// Library song ids: `yt_<youtube id>`, stable across re-harvests (favourites and recents keep working).
pub const ID_PREFIX: &str = "yt_";

static LIBRARY: OnceLock<Songbook> = OnceLock::new();

/// Songs from `assets/library.json`, in file order (channel, then code).
pub fn parse(json: &str) -> Result<Vec<Song>, serde_json::Error> {
    let file: LibraryFile = serde_json::from_str(json)?;
    let songs = file.channels.into_iter().flat_map(|channel| {
        let (name, intro_skip_secs) = (channel.name, channel.intro_skip_secs);
        channel.songs.into_iter().map(move |LibraryRow(youtube_id, code, duration_secs, title, artist, alias)| Song {
            id: format!("{ID_PREFIX}{youtube_id}"),
            code: code.to_string(),
            title,
            artist,
            aliases: if alias.is_empty() { Vec::new() } else { vec![alias] },
            youtube_id,
            guide: None,
            duration_secs,
            // A short song must still start before its end
            intro_skip_secs: intro_skip_secs.min(duration_secs / 4),
            category: String::new(),
            channel: name.clone(),
            is_favorite: false,
        })
    });
    Ok(songs.collect())
}

/// Parse and keep the library for the rest of the page's life; a second call is ignored.
pub fn install(json: &str) -> Result<Library, serde_json::Error> {
    let book = Songbook::new(parse(json)?);
    Ok(Library::new(LIBRARY.get_or_init(|| book)))
}

/// The library once installed, else empty.
pub fn loaded() -> Library {
    LIBRARY.get().map_or_else(Library::default, Library::new)
}
