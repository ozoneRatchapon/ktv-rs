use super::{layout::retype, normalize::{normalize, search_key}};
use crate::library::Library;
use crate::types::Song;

/// Songs matching a query, in the order given.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHits<'a> {
    pub songs: Vec<&'a Song>,
    /// Set when nothing matched as typed but the query retyped on the other keyboard layout did.
    pub retyped: Option<String>,
}

fn matching<'a>(songs: impl IntoIterator<Item = &'a Song>, library: Library, query: &str) -> Vec<&'a Song> {
    let needle = normalize(query);
    let code = query.trim();
    if needle.is_empty() {
        return songs.into_iter().collect();
    }
    songs
        .into_iter()
        .filter(|s| {
            s.code.contains(code)
                || match library.search_key(s) {
                    Some(key) => key.contains(&needle),
                    None => search_key(s).contains(&needle),
                }
        })
        .collect()
}

/// Search titles, artists, aliases and keypad codes; retries on the other keyboard layout when empty.
/// `library` supplies precomputed keys for its own songs (others are normalised on the fly).
pub fn search<'a, I>(songs: I, library: Library, query: &str) -> SearchHits<'a>
where
    I: IntoIterator<Item = &'a Song> + Clone,
{
    let hits = matching(songs.clone(), library, query);
    if !hits.is_empty() {
        return SearchHits { songs: hits, retyped: None };
    }
    match retype(query).map(|q| (matching(songs, library, &q), q)) {
        Some((found, q)) if !found.is_empty() => SearchHits { songs: found, retyped: Some(q) },
        _ => SearchHits { songs: hits, retyped: None },
    }
}
