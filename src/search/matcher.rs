use super::{layout::retype, normalize::normalize};
use crate::types::Song;

/// Songs matching a query, in catalog order.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHits {
    pub songs: Vec<Song>,
    /// Set when nothing matched as typed but the query retyped on the other keyboard layout did.
    pub retyped: Option<String>,
}

fn matching(songs: &[Song], query: &str) -> Vec<Song> {
    let needle = normalize(query);
    let code = query.trim();
    if needle.is_empty() {
        return songs.to_vec();
    }
    songs
        .iter()
        .filter(|s| {
            s.code.contains(code)
                || normalize(&s.title).contains(&needle)
                || normalize(&s.artist).contains(&needle)
                || s.aliases.iter().any(|a| normalize(a).contains(&needle))
        })
        .cloned()
        .collect()
}

/// Search titles, artists, aliases and keypad codes; retries on the other keyboard layout when empty.
pub fn search(songs: &[Song], query: &str) -> SearchHits {
    let hits = matching(songs, query);
    if !hits.is_empty() {
        return SearchHits { songs: hits, retyped: None };
    }
    match retype(query).map(|q| (matching(songs, &q), q)) {
        Some((found, q)) if !found.is_empty() => SearchHits { songs: found, retyped: Some(q) },
        _ => SearchHits { songs: hits, retyped: None },
    }
}
