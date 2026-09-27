use std::sync::LazyLock;

use crate::types::Song;

/// Thai marks people often skip or mistype: mai taikhu (U+0E47), the four tone marks (U+0E48-U+0E4B),
/// thanthakhat (U+0E4C) and nikhahit (U+0E4D).
fn is_thai_mark(c: char) -> bool {
    ('\u{0E47}'..='\u{0E4D}').contains(&c)
}

/// Thai block U+0E00-U+0E7F: which characters [`normalize`] keeps (alphanumeric, not a mark), worked out
/// once from the same rules; a table lookup instead of a Unicode property search per character.
const THAI_BLOCK: usize = 0x0E00;
static THAI_KEPT: LazyLock<[bool; 0x80]> = LazyLock::new(|| {
    std::array::from_fn(|i| char::from_u32((THAI_BLOCK + i) as u32).is_some_and(|c| c.is_alphanumeric() && !is_thai_mark(c)))
});

/// Lower-case letters and digits only: spaces, punctuation and Thai tone marks dropped, so
/// "รักไม่ไหว", "รักไมไหว" and "Rak-Mai Wai" compare the way people expect.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    push_normalized(&mut out, text);
    out
}

/// [`normalize`] into `out`. Fast paths (same result): ASCII needs no Unicode tables, and Thai has no case.
fn push_normalized(out: &mut String, text: &str) {
    for c in text.chars() {
        if c.is_ascii() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
            }
        } else if let Some(&keep) = THAI_KEPT.get((c as usize).wrapping_sub(THAI_BLOCK)) {
            if keep {
                out.push(c);
            }
        } else if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        }
    }
}

/// Everything a query is matched against, normalised: title, artist and aliases. A line break keeps
/// fields apart ([`normalize`] never outputs one, so a match cannot run across two fields).
pub fn search_key(song: &Song) -> String {
    let fields = [&song.title, &song.artist].into_iter().chain(&song.aliases);
    let mut key = String::with_capacity(song.title.len() + song.artist.len() + 8);
    for (i, field) in fields.enumerate() {
        if i > 0 {
            key.push('\n');
        }
        push_normalized(&mut key, field);
    }
    key
}
