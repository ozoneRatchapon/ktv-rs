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

/// Inverse of [`percent_encode`] (also reads `+` as a space). `None` for a broken escape or bytes that are not UTF-8.
pub fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}
