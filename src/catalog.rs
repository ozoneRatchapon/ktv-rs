use std::sync::LazyLock;

use crate::types::Song;

/// Song data lives in `assets/catalog.json` (edited by hand; guide timings come from the in-app
/// Guide Timing Tools' "Copy JSON").
/// Embedded at compile time: no startup fetch, and `cargo test` rejects a malformed file.
/// Guide timing is lined up by ear, not guessed: MV time = offset_secs + rate * karaoke time.
pub const CATALOG_JSON: &str = include_str!("../assets/catalog.json");

static CATALOG: LazyLock<Vec<Song>> =
    LazyLock::new(|| serde_json::from_str(CATALOG_JSON).expect("assets/catalog.json must match Vec<Song>"));

pub fn builtin_catalog() -> &'static [Song] {
    &CATALOG
}

pub fn get_initial_catalog() -> Vec<Song> {
    CATALOG.to_vec()
}

pub fn get_categories() -> Vec<&'static str> {
    vec!["All", "Rock", "Pop", "Indie", "Modern", "Luk Thung", "Classic 90s"]
}

/// Keypad codes reserved for songs added by URL; built-in songs stay below this range.
pub const CUSTOM_CODES: std::ops::RangeInclusive<u32> = 90001..=99999;

/// Lowest reserved code not taken by any song in `catalog` (`None` once all 9999 are used).
pub fn next_custom_code(catalog: &[Song]) -> Option<String> {
    let taken: std::collections::HashSet<&str> = catalog.iter().map(|s| s.code.as_str()).collect();
    CUSTOM_CODES.map(|c| c.to_string()).find(|c| !taken.contains(c.as_str()))
}

/// Every code in `CUSTOM_CODES` is taken, so a new custom song cannot get a keypad code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomCodesExhausted;

impl std::fmt::Display for CustomCodesExhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (first, last) = (CUSTOM_CODES.start(), CUSTOM_CODES.end());
        write!(f, "all custom song codes ({first}-{last}) are in use")
    }
}

impl std::error::Error for CustomCodesExhausted {}

/// Add a custom song, or refresh it if the same video was added before (it keeps its code).
/// Returns the stored song; fails only for a new video once the reserved code range is exhausted.
pub fn upsert_custom(catalog: &mut Vec<Song>, mut song: Song) -> Result<Song, CustomCodesExhausted> {
    match catalog.iter_mut().find(|s| s.id == song.id) {
        Some(existing) => {
            song.code = existing.code.clone();
            *existing = song.clone();
        }
        None => {
            song.code = next_custom_code(catalog).ok_or(CustomCodesExhausted)?;
            catalog.push(song.clone());
        }
    }
    Ok(song)
}
