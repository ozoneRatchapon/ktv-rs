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

/// The straightforward definition `normalize` / `search_key` must keep matching (they take fast paths).
fn reference_normalize(text: &str) -> String {
    let thai_mark = |c: char| ('\u{0E47}'..='\u{0E4D}').contains(&c);
    text.chars().filter(|c| c.is_alphanumeric() && !thai_mark(*c)).flat_map(char::to_lowercase).collect()
}

#[test]
fn test_search_key_matches_reference_on_every_song() {
    let library = app::library::parse(include_str!("../assets/library.json")).expect("library parses");
    let songs = app::catalog::builtin_catalog().iter().chain(&library);
    for song in songs {
        let fields = [&song.title, &song.artist].into_iter().chain(&song.aliases);
        let reference = fields.map(|f| reference_normalize(f)).collect::<Vec<_>>().join("\n");
        assert_eq!(app::search::search_key(song), reference, "{} - {}", song.title, song.artist);
    }
    for text in ["Rak-Mai Wai", "รักไม่ไหว", "ÄÖÜ Straße", "İstanbul", "ΣΊΣΥΦΟΣ", "１２３", "Ⅻ"] {
        assert_eq!(app::search::normalize(text), reference_normalize(text), "{text}");
    }
}

#[test]
fn test_substring_edits_counts_typos_inside_a_longer_key() {
    use app::search::substring_edits;
    let mut row = Vec::new();
    let chars = |s: &str| s.chars().collect::<Vec<_>>();
    assert_eq!(substring_edits(&chars("bodyslam"), "เพลง\nbodyslam", 2, &mut row), Some(0));
    assert_eq!(substring_edits(&chars("bodyslan"), "เพลง\nbodyslam", 2, &mut row), Some(1), "changed letter");
    assert_eq!(substring_edits(&chars("bodslam"), "x\nbodyslam", 2, &mut row), Some(1), "missing letter");
    assert_eq!(substring_edits(&chars("boddyslam"), "bodyslam", 2, &mut row), Some(1), "extra letter");
    assert_eq!(substring_edits(&chars("zzzzzz"), "bodyslam", 1, &mut row), None);
}

#[test]
fn test_typo_search_finds_the_song_closest_first() {
    use app::search::allowed_edits;
    let cat = builtin_catalog();
    // รักไม่ไหวแล้วโว้ย with ห left out
    let hits = search(cat.iter(), Library::default(), "รักไม่ไวแล้ว");
    assert!(hits.fuzzy);
    assert_eq!(hits.songs[0].title, "รักไม่ไหวแล้วโว้ย");
    // An exact match never turns fuzzy, and short queries must match as typed
    assert!(!search(cat.iter(), Library::default(), "รักไม่ไหว").fuzzy);
    assert_eq!(allowed_edits(3), 0);
    let short = search(cat.iter(), Library::default(), "zzq");
    assert!(short.songs.is_empty() && !short.fuzzy);
}

#[test]
fn test_typo_search_over_full_library_is_fast() {
    let library = app::library::parse(include_str!("../assets/library.json")).expect("library parses");
    let book: &'static app::library::Songbook = Box::leak(Box::new(app::library::Songbook::new(library)));
    let lib = Library::new(book);
    let all = builtin_catalog().iter().chain(lib.songs());
    let start = std::time::Instant::now();
    let hits = search(all, lib, "bodyslan");
    let elapsed = start.elapsed();
    assert!(hits.fuzzy && hits.songs.iter().any(|s| s.artist.to_lowercase().contains("bodyslam")));
    // Runs only after the exact and other-layout passes found nothing, once per keystroke
    assert!(elapsed.as_millis() < 250, "typo search over {} songs took {elapsed:?}", lib.songs().len());
}
