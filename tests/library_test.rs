use std::collections::HashSet;
use std::sync::LazyLock;

use app::catalog::{builtin_catalog, find_song, keypad_code, CUSTOM_CODES};
use app::library::{self, Library, Songbook, ID_PREFIX};
use app::picks::{Picks, Shelf};
use app::search::search;
use app::types::Song;

const LIBRARY_JSON: &str = include_str!("../assets/library.json");
const MV_GUIDES_JSON: &str = include_str!("../assets/mv_guides.json");

/// Parsed once and leaked, as the app keeps it for the page's life (guides first, as the app installs them).
static BOOK: LazyLock<&'static Songbook> = LazyLock::new(|| {
    library::install_guides(MV_GUIDES_JSON).expect("assets/mv_guides.json must match MvGuide entries");
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
        assert_eq!(keypad_code(&song.code), Some(song.code.as_str()), "{}: code must be 5 digits", song.id);
        assert!(!song.title.trim().is_empty() && !song.artist.trim().is_empty(), "{}: empty title/artist", song.id);
        assert!(song.intro_skip_secs < song.duration_secs, "{}: intro skip past the end", song.id);
        assert_eq!(song.guide, app::library::timed_guide(&song.youtube_id), "{}: a guide only from a timed mv_guides entry", song.id);
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
    let rock = picks.shelf(cat, lib(), &Shelf::Category("Rock"));
    assert!(rock.iter().all(|s| s.category == "Rock"));
    assert!(rock.iter().any(|s| s.id.starts_with(ID_PREFIX)), "labelled library songs join the genre shelf");

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

#[test]
fn test_songbook_video_finds_curated_and_library_but_not_custom() {
    use app::catalog::{custom_song_id, songbook_video};
    let curated = &builtin_catalog()[0];
    let listed = &SONGS[0];
    let custom = Song { id: custom_song_id("zzzzzzzzzzz"), youtube_id: "zzzzzzzzzzz".to_string(), ..curated.clone() };
    let catalog: Vec<Song> = builtin_catalog().iter().cloned().chain([custom]).collect();

    assert_eq!(songbook_video(&catalog, lib(), &curated.youtube_id).map(|s| &s.id), Some(&curated.id));
    assert_eq!(songbook_video(&catalog, lib(), &listed.youtube_id).map(|s| &s.id), Some(&listed.id));
    assert_eq!(songbook_video(&catalog, lib(), "zzzzzzzzzzz"), None, "an Add URL song is not a songbook song");
    assert_eq!(songbook_video(&catalog, Library::default(), &listed.youtube_id), None, "library not loaded yet");
}

#[test]
fn test_retired_codes_are_not_reused() {
    let retired: std::collections::HashMap<String, String> =
        serde_json::from_str(include_str!("../tools/retired_codes.json")).expect("tools/retired_codes.json must parse");
    for song in *SONGS {
        if let Some(vid) = retired.get(&song.code) {
            assert_eq!(vid, &song.youtube_id, "code {} was retired from {vid}, now given to {}", song.code, song.title);
        }
    }
}

#[test]
fn test_mv_guides_point_at_library_songs() {
    use app::library::{guides, suggested_video, timed_guide};
    let by_video: std::collections::HashMap<&str, &Song> = SONGS.iter().map(|s| (s.youtube_id.as_str(), s)).collect();
    let guides = guides().expect("installed with the library");
    assert!(!guides.is_empty());
    for (karaoke_id, guide) in guides {
        let song = by_video.get(karaoke_id.as_str()).unwrap_or_else(|| panic!("{karaoke_id}: not a library song"));
        assert!(is_youtube_id(&guide.video_id), "{karaoke_id}: bad video id {}", guide.video_id);
        assert_ne!(&guide.video_id, karaoke_id, "{karaoke_id}: the guide must be a different video");
        match guide.offset_secs {
            // Timed: the library song carries it, so its Vocal button shows
            Some(_) => {
                assert_eq!(song.guide, timed_guide(karaoke_id), "{karaoke_id}");
                assert_eq!(suggested_video(karaoke_id), None);
            }
            None => {
                assert_eq!(song.guide, None, "{karaoke_id}: an unchecked suggestion must not reach singers");
                assert_eq!(suggested_video(karaoke_id), Some(guide.video_id.as_str()));
            }
        }
    }
}

#[test]
fn test_every_label_keeps_its_code_range() {
    let ranges = [
        ("GMM Karaoke", 30001..=49999),
        ("Whattheduck", 50001..=54999),
        ("Muzik Move Karaoke", 55001..=59999),
        ("RS Music", 60001..=64999),
        ("Smallroom Karaoke", 65001..=69999),
    ];
    for (channel, range) in ranges {
        let songs: Vec<&Song> = SONGS.iter().filter(|s| s.channel == channel).collect();
        assert!(!songs.is_empty(), "{channel}: no songs");
        for song in songs {
            let code: u32 = song.code.parse().unwrap();
            assert!(range.contains(&code), "{channel}: {} has code {code}", song.title);
        }
    }
}

#[test]
fn test_library_genres_are_known_and_label_based() {
    use app::catalog::CATEGORIES;
    let file: app::library::LibraryFile = serde_json::from_str(LIBRARY_JSON).unwrap();
    for channel in &file.channels {
        let ids: HashSet<&str> = channel.songs.iter().map(|row| row.0.as_str()).collect();
        for (vid, genre) in &channel.genres {
            assert!(ids.contains(vid.as_str()), "{}: genre for unknown video {vid}", channel.name);
            assert!(CATEGORIES.contains(&genre.as_str()), "{}: unknown genre {genre}", channel.name);
        }
    }
    for song in *SONGS {
        assert!(song.category.is_empty() || CATEGORIES.contains(&song.category.as_str()), "{}: {}", song.title, song.category);
    }
    let count = |channel: &str, genre: &str| SONGS.iter().filter(|s| s.channel == channel && s.category == genre).count();
    let smallroom = SONGS.iter().filter(|s| s.channel == "Smallroom Karaoke").count();
    assert_eq!(count("Smallroom Karaoke", "Indie"), smallroom, "Smallroom is an indie label");
    assert!(count("GMM Karaoke", "Luk Thung") > 1000 && count("GMM Karaoke", "Rock") > 300);
}

#[test]
fn test_genre_chip_now_lists_library_songs() {
    let shelf = Picks::default().shelf(builtin_catalog(), lib(), &Shelf::Category("Luk Thung"));
    let from_library = shelf.iter().filter(|s| s.id.starts_with(ID_PREFIX)).count();
    assert!(from_library > 1000, "{from_library} library songs under Luk Thung");
}

#[test]
fn test_auto_timed_guides_are_gmm_official_audio_at_the_intro_offset() {
    use app::library::{auto_timed, guides};
    let _ = *SONGS;
    for (karaoke_id, guide) in guides().expect("installed with the library").iter().filter(|(_, g)| g.auto) {
        let song = SONGS.iter().find(|s| &s.youtube_id == karaoke_id).unwrap();
        assert_eq!(song.channel, "GMM Karaoke", "{karaoke_id}: only GMM has the fixed intro");
        assert_eq!((guide.offset_secs, guide.rate), (Some(-18.2), Some(1.0)), "{karaoke_id}");
        assert!(auto_timed(karaoke_id));
        assert!(song.guide.is_some(), "{karaoke_id}: an auto-timed song has its Vocal button");
    }
}

#[test]
fn test_song_index_for_the_phone_page_has_every_song_once_with_four_fields() {
    use app::catalog::builtin_catalog;
    let library = app::library::parse(include_str!("../assets/library.json")).expect("library parses");
    let lib = app::library::Library::new(Box::leak(Box::new(app::library::Songbook::new(library))));
    let songs: Vec<_> = builtin_catalog().iter().chain(lib.songs()).collect();
    let index = app::library::song_index(songs.iter().copied());
    let lines: Vec<&str> = index.lines().collect();
    assert_eq!(lines.len(), songs.len());
    assert!(lines.iter().all(|line| line.split('\t').count() == 4), "four fields per line");
    let codes: std::collections::HashSet<&str> = lines.iter().map(|l| l.split('\t').next().unwrap_or_default()).collect();
    assert_eq!(codes.len(), lines.len(), "codes are unique, so a tapped result requests one song");
    assert!(index.contains("10004\tรักไม่ไหวแล้วโว้ย\tโจอี้ ภูวศิษฐ์\t"), "booth codes and titles");
}
