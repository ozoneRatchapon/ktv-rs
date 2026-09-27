use super::fuzzy::{allowed_edits, substring_edits};
use super::{layout::retype, normalize::{normalize, search_key}};
use crate::library::Library;
use crate::types::Song;

/// Songs matching a query, in the order given.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHits<'a> {
    pub songs: Vec<&'a Song>,
    /// Set when nothing matched as typed but the query retyped on the other keyboard layout did.
    pub retyped: Option<String>,
    /// Nothing matched exactly: these are close matches (a typo or two), closest first.
    pub fuzzy: bool,
}

fn key_of<'s>(library: Library, song: &'s Song, scratch: &'s mut String) -> &'s str {
    match library.search_key(song) {
        Some(key) => key,
        None => {
            *scratch = search_key(song);
            scratch
        }
    }
}

/// Songs within the allowed number of typos of the query, fewest first (ties keep the given order).
fn close<'a>(songs: impl IntoIterator<Item = &'a Song>, library: Library, query: &str) -> Vec<&'a Song> {
    let needle: Vec<char> = normalize(query).chars().collect();
    let max = allowed_edits(needle.len());
    if max == 0 {
        return Vec::new();
    }
    let (mut row, mut scratch) = (Vec::with_capacity(needle.len() + 1), String::new());
    let mut found: Vec<(usize, &Song)> = songs
        .into_iter()
        .filter_map(|s| substring_edits(&needle, key_of(library, s, &mut scratch), max, &mut row).map(|d| (d, s)))
        .collect();
    found.sort_by_key(|(d, _)| *d);
    found.into_iter().map(|(_, s)| s).collect()
}

/// How well a song matched, best first: its code, then the title (whole, start, inside), the artist (start,
/// inside), an alias, and last a query whose words match separately ("artist title" typed in one box).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Code,
    TitleWhole,
    TitleStart,
    TitleInside,
    ArtistStart,
    ArtistInside,
    Alias,
    Words,
}

/// `key` is [`search_key`]: title, artist, then aliases, one per line.
fn rank(key: &str, needle: &str, words: &[String]) -> Option<Rank> {
    let mut fields = key.split('\n');
    let (title, artist) = (fields.next().unwrap_or_default(), fields.next().unwrap_or_default());
    let rank = match () {
        _ if title == needle => Rank::TitleWhole,
        _ if title.starts_with(needle) => Rank::TitleStart,
        _ if title.contains(needle) => Rank::TitleInside,
        _ if artist.starts_with(needle) => Rank::ArtistStart,
        _ if artist.contains(needle) => Rank::ArtistInside,
        _ if fields.any(|alias| alias.contains(needle)) => Rank::Alias,
        _ if words.len() > 1 && words.iter().all(|w| key.contains(w.as_str())) => Rank::Words,
        _ => return None,
    };
    Some(rank)
}

fn matching<'a>(songs: impl IntoIterator<Item = &'a Song>, library: Library, query: &str) -> Vec<&'a Song> {
    let needle = normalize(query);
    let code = query.trim();
    if needle.is_empty() {
        return songs.into_iter().collect();
    }
    let words: Vec<String> = query.split_whitespace().map(normalize).filter(|w| !w.is_empty()).collect();
    let mut scratch = String::new();
    let mut ranked: Vec<(Rank, &Song)> = songs
        .into_iter()
        .filter_map(|s| match s.code.contains(code) {
            true => Some((Rank::Code, s)),
            false => rank(key_of(library, s, &mut scratch), &needle, &words).map(|r| (r, s)),
        })
        .collect();
    // Stable: equal ranks keep the given order (curated catalog first, then the library)
    ranked.sort_by_key(|(r, _)| *r);
    ranked.into_iter().map(|(_, s)| s).collect()
}

/// Search titles, artists, aliases and keypad codes, best match first; when nothing matches, retries on the other keyboard layout,
/// then allows a typo or two ([`super::fuzzy`]).
/// `library` supplies precomputed keys for its own songs (others are normalised on the fly).
pub fn search<'a, I>(songs: I, library: Library, query: &str) -> SearchHits<'a>
where
    I: IntoIterator<Item = &'a Song> + Clone,
{
    let hits = matching(songs.clone(), library, query);
    if !hits.is_empty() {
        return SearchHits { songs: hits, retyped: None, fuzzy: false };
    }
    if let Some((found, q)) = retype(query).map(|q| (matching(songs.clone(), library, &q), q)) {
        if !found.is_empty() {
            return SearchHits { songs: found, retyped: Some(q), fuzzy: false };
        }
    }
    let near = close(songs, library, query);
    let fuzzy = !near.is_empty();
    SearchHits { songs: near, retyped: None, fuzzy }
}
