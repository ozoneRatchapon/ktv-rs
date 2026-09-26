use std::collections::HashSet;
use std::sync::LazyLock;

use app::catalog::{builtin_catalog, find_song, CUSTOM_CODES};
use app::library::{self, Library, Songbook, ID_PREFIX};
use app::picks::{Picks, Shelf};
use app::search::search;
use app::types::Song;

const LIBRARY_JSON: &str = include_str!("../assets/library.json");

/// Parsed once and leaked, as the app keeps it for the page's life.
static BOOK: LazyLock<&'static Songbook> = LazyLock::new(|| {
    Box::leak(Box::new(Songbook::new(library::parse(LIBRARY_JSON).expect("assets/library.json must parse"))))
});
static SONGS: LazyLock<&'static [Song]> = LazyLock::new(|| lib().songs());

fn lib() -> Library {
    Library::new(*BOOK)
}

fn is_youtube_id(id: &str) -> bool {
    id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[test]
fn test_library_is_well_formed() {
    assert!(SONGS.len() > 1000, "library looks truncated: {} songs", SONGS.len());
    for song in *SONGS {
        assert!(song.id.starts_with(ID_PREFIX), "{}: id prefix", song.id);
        assert!(is_youtube_id(&song.youtube_id), "{}: bad youtube_id", song.id);
        assert!(song.code.len() == 5 && song.code.bytes().all(|b| b.is_ascii_digit()), "{}: code must be 5 digits", song.id);
        assert!(!song.title.trim().is_empty() && !song.artist.trim().is_empty(), "{}: empty title/artist", song.id);
        assert!(song.intro_skip_secs < song.duration_secs, "{}: intro skip past the end", song.id);
        assert!(song.guide.is_none(), "{}: library songs have no guide timing", song.id);
    }
}

#[test]
fn test_library_ids_and_codes_never_collide_with_the_catalog_or_add_url() {
    let mut ids: HashSet<&str> = builtin_catalog().iter().map(|s| s.id.as_str()).collect();
    let mut codes: HashSet<&str> = builtin_catalog().iter().map(|s| s.code.as_str()).collect();
    let curated_videos: HashSet<&str> = builtin_catalog().iter().map(|s| s.youtube_id.as_str()).collect();
    for song in *SONGS {
        assert!(ids.insert(&song.id), "duplicate id {}", song.id);
        assert!(codes.insert(&song.code), "duplicate code {}", song.code);
        assert!(!curated_videos.contains(song.youtube_id.as_str()), "{}: already in catalog.json", song.id);
        let code: u32 = song.code.parse().unwrap();
        assert!(!CUSTOM_CODES.contains(&code), "{}: code {code} is in the Add URL range", song.id);
    }
}

#[test]
fn test_code_lookup_prefers_the_catalog_then_the_library() {
    let song = &SONGS[SONGS.len() / 2];
    let found = find_song(builtin_catalog(), lib(), |s| s.code == song.code).unwrap();
    assert_eq!(found.id, song.id);
    let curated = &builtin_catalog()[0];
    assert_eq!(find_song(builtin_catalog(), lib(), |s| s.code == curated.code).unwrap().id, curated.id);
    assert!(find_song(builtin_catalog(), Library::default(), |s| s.code == song.code).is_none(), "not loaded yet");
}

#[test]
fn test_shelves_and_search_cover_the_library() {
    let cat = builtin_catalog();
    let mut picks = Picks::default();
    let starred = &SONGS[10];
    picks.toggle_favourite(&starred.id);
    picks.record_sung(&starred.id, 60.0);
    assert_eq!(picks.shelf(cat, lib(), &Shelf::All).len(), cat.len() + SONGS.len());
    assert_eq!(picks.shelf(cat, lib(), &Shelf::Favourites)[0].id, starred.id);
    assert_eq!(picks.shelf(cat, lib(), &Shelf::Recent)[0].id, starred.id);
    assert!(picks.shelf(cat, lib(), &Shelf::Category("Rock")).iter().all(|s| !s.id.starts_with(ID_PREFIX)));

    let all = cat.iter().chain(*SONGS);
    let hits = search(all.clone(), lib(), &starred.title);
    assert!(hits.songs.iter().any(|s| s.id == starred.id), "title search finds a library song");
    let with_alias = SONGS.iter().find(|s| !s.aliases.is_empty()).unwrap();
    assert!(search(all, lib(), &with_alias.aliases[0]).songs.iter().any(|s| s.id == with_alias.id), "romanised alias");
}

#[test]
fn test_precomputed_key_only_for_the_library_s_own_entries() {
    let song = &SONGS[42];
    assert_eq!(lib().search_key(song), Some(app::search::search_key(song).as_str()));
    assert_eq!(lib().search_key(&song.clone()), None, "a copy (e.g. in the queue) is normalised on the fly");
    assert_eq!(lib().search_key(&builtin_catalog()[0]), None);
    assert_eq!(Library::default().search_key(song), None);
}

#[test]
fn test_malformed_library_is_an_error_not_a_panic() {
    assert!(library::parse("{}").is_err());
    assert!(library::parse(r#"{"channels":[{"name":"x","intro_skip_secs":0,"songs":[["id",1]]}]}"#).is_err());
    let one = r#"{"channels":[{"name":"GMM Karaoke","intro_skip_secs":18,"songs":[["abcdefghijk",30001,64,"t","a",""]]}]}"#;
    let songs = library::parse(one).unwrap();
    assert_eq!((songs[0].intro_skip_secs, songs[0].aliases.len()), (16, 0), "intro capped on a short song");
}
