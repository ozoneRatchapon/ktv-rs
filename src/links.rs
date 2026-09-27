//! Outbound links built from song data (plain `<a href>`: nothing is fetched in the background).

use crate::types::Song;

/// RFC 3986 percent-encoding of everything but unreserved characters (UTF-8 bytes for Thai).
pub fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A web search for the song's chords ("คอร์ด <title> <artist>"). KTV-RS ships no chord sheets: they are the chord
/// sites' own work, and the song's audio cannot be analysed, so musicians are sent to the sites that have them.
pub fn chords_search_url(song: &Song) -> String {
    let query = format!("คอร์ด {} {}", song.title, song.artist);
    format!("https://www.google.com/search?q={}", percent_encode(&query))
}
