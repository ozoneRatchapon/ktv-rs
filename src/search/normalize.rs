use crate::types::Song;

/// Thai marks people often skip or mistype: mai taikhu (U+0E47), the four tone marks (U+0E48-U+0E4B),
/// thanthakhat (U+0E4C) and nikhahit (U+0E4D).
fn is_thai_mark(c: char) -> bool {
    ('\u{0E47}'..='\u{0E4D}').contains(&c)
}

/// Lower-case letters and digits only: spaces, punctuation and Thai tone marks dropped, so
/// "รักไม่ไหว", "รักไมไหว" and "Rak-Mai Wai" compare the way people expect.
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() && !is_thai_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Everything a query is matched against, normalised: title, artist and aliases. A line break keeps
/// fields apart ([`normalize`] never outputs one, so a match cannot run across two fields).
pub fn search_key(song: &Song) -> String {
    let fields = [&song.title, &song.artist].into_iter().chain(&song.aliases);
    fields.map(|f| normalize(f)).collect::<Vec<_>>().join("\n")
}
