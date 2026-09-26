use app::catalog::builtin_catalog;
use app::library::Library;
use app::search::{normalize, retype, search};

fn titles(query: &str) -> Vec<String> {
    search(builtin_catalog(), Library::default(), query).songs.into_iter().map(|s| s.title.clone()).collect()
}

#[test]
fn test_normalize_drops_tone_marks_spacing_and_case() {
    assert_eq!(normalize("รักไม่ไหว"), normalize("รักไมไหว"));
    assert_eq!(normalize("Rak-Mai Wai"), "rakmaiwai");
    assert_eq!(normalize("ยิ่งใกล้...ยิ่งไกล"), normalize("ยิงใกล ยิงไกล"));
}

#[test]
fn test_thai_without_tone_marks_or_spaces_finds_the_song() {
    assert_eq!(titles("รักไมไหวแลวโวย"), ["รักไม่ไหวแล้วโว้ย"]);
    assert_eq!(titles("ยิ่งใกล้ ยิ่งไกล"), ["ยิ่งใกล้...ยิ่งไกล"]);
}

#[test]
fn test_romanised_alias_finds_the_thai_title() {
    assert_eq!(titles("rak mai wai"), ["รักไม่ไหวแล้วโว้ย"]);
    assert_eq!(titles("RAK-MAI-WAI"), ["รักไม่ไหวแล้วโว้ย"]);
}

#[test]
fn test_code_title_and_artist_still_match() {
    assert_eq!(titles("10004"), ["รักไม่ไหวแล้วโว้ย"]);
    assert!(titles("cocktail").len() >= 5, "artist, case-insensitive");
    assert_eq!(titles("").len(), builtin_catalog().len());
}

#[test]
fn test_retype_kedmanee_both_ways() {
    assert_eq!(retype("l;ylfu").as_deref(), Some("สวัสดี"), "Thai typed on the US layout");
    assert_eq!(retype("นฟหรห").as_deref(), Some("oasis"), "English typed on the Thai layout");
    assert_eq!(retype("12345"), Some("ๅ/-ภถ".to_string()));
}

#[test]
fn test_wrong_layout_query_is_retried_only_when_nothing_matches() {
    // "รัก" typed with the keyboard on English: i y d
    let hits = search(builtin_catalog(), Library::default(), "iyd");
    assert_eq!(hits.retyped.as_deref(), Some("รัก"));
    assert!(hits.songs.iter().all(|s| s.title.contains("รัก")));
    let hits = search(builtin_catalog(), Library::default(), "นฟหรห");
    assert_eq!(hits.retyped.as_deref(), Some("oasis"));
    assert_eq!(hits.songs.len(), 1);
    let direct = search(builtin_catalog(), Library::default(), "oasis");
    assert_eq!((direct.retyped, direct.songs.len()), (None, 1), "a query that matches is never retyped");
}

#[test]
fn test_no_match_stays_empty() {
    let hits = search(builtin_catalog(), Library::default(), "zzzzqqq");
    assert!(hits.songs.is_empty());
    assert_eq!(hits.retyped, None);
}
