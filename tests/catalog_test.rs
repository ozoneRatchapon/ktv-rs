use std::collections::HashSet;

use app::catalog::{builtin_catalog, get_initial_catalog, CATEGORIES, upsert_custom, CustomCodesExhausted, CUSTOM_CODES};

#[test]
fn test_guide_is_a_different_video_than_karaoke() {
    for song in get_initial_catalog() {
        if let Some(guide) = &song.guide {
            assert_ne!(
                guide.video_id, song.youtube_id,
                "{} ({}): guide must be the original-singer track, not the karaoke video",
                song.code, song.title
            );
        }
    }
}

#[test]
fn test_guide_timing_within_sync_controller_range() {
    for song in get_initial_catalog() {
        if let Some(guide) = &song.guide {
            // Controller can only nudge YouTube playback rate by 0.95..1.05
            assert!(
                (0.95..=1.05).contains(&guide.rate),
                "{}: rate {} outside controllable range",
                song.code, guide.rate
            );
            assert!(
                guide.offset_secs.abs() <= 150.0,
                "{}: offset {} exceeds measured search window",
                song.code, guide.offset_secs
            );
        }
    }
}

fn is_youtube_id(id: &str) -> bool {
    id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[test]
fn test_catalog_json_loads_and_is_well_formed() {
    let catalog = get_initial_catalog();
    assert!(!catalog.is_empty(), "assets/catalog.json is empty");
    let categories: HashSet<&str> = CATEGORIES.into_iter().collect();
    let mut ids = HashSet::new();
    let mut codes = HashSet::new();
    for song in &catalog {
        assert!(ids.insert(song.id.as_str()), "duplicate id {}", song.id);
        assert!(codes.insert(song.code.as_str()), "duplicate code {}", song.code);
        assert!(song.code.len() == 5 && song.code.bytes().all(|b| b.is_ascii_digit()), "{}: code must be 5 digits", song.id);
        assert!(categories.contains(song.category.as_str()), "{}: unknown category {}", song.code, song.category);
        assert!(is_youtube_id(&song.youtube_id), "{}: bad youtube_id {}", song.code, song.youtube_id);
        assert!(song.intro_skip_secs < song.duration_secs, "{}: intro skip past the end", song.code);
        if let Some(guide) = &song.guide {
            assert!(is_youtube_id(&guide.video_id), "{}: bad guide id {}", song.code, guide.video_id);
        }
    }
}

#[test]
fn test_default_queue_indices_exist() {
    // main.rs seeds the player and queue with catalog[2], [3], [6], [12]
    assert!(get_initial_catalog().len() > 12);
}

fn custom(vid: &str) -> app::types::Song {
    app::types::Song { id: format!("custom_{vid}"), code: String::new(), youtube_id: vid.to_string(), guide: None, ..builtin_catalog()[0].clone() }
}

#[test]
fn test_builtin_codes_stay_below_custom_range() {
    for song in builtin_catalog() {
        let code: u32 = song.code.parse().unwrap();
        assert!(code < *CUSTOM_CODES.start(), "{}: code {code} is in the custom range", song.id);
    }
}

#[test]
fn test_upsert_custom_assigns_unique_codes_and_reuses_on_readd() {
    let mut catalog = builtin_catalog().to_vec();
    let a = upsert_custom(&mut catalog, custom("dQw4w9WgXcQ")).unwrap();
    let b = upsert_custom(&mut catalog, custom("9bZkp7q19f0")).unwrap();
    assert_eq!((a.code.as_str(), b.code.as_str()), ("90001", "90002"));

    let renamed = app::types::Song { title: "Renamed".to_string(), ..custom("dQw4w9WgXcQ") };
    let again = upsert_custom(&mut catalog, renamed).unwrap();
    assert_eq!(again.code, "90001");
    assert_eq!(catalog.len(), builtin_catalog().len() + 2);
    assert!(catalog.iter().any(|s| s.title == "Renamed"));
}

#[test]
fn test_start_sec_honours_intro_choice() {
    let song = get_initial_catalog().into_iter().find(|s| s.intro_skip_secs > 0).expect("a song with an intro");
    assert_eq!(song.start_sec(true), u64::from(song.intro_skip_secs));
    assert_eq!(song.start_sec(false), 0);
}

#[test]
fn test_upsert_custom_errors_when_codes_exhausted_but_readd_still_works() {
    let mut catalog: Vec<_> = CUSTOM_CODES
        .map(|c| app::types::Song { code: c.to_string(), ..custom(&format!("id{c:07}")) })
        .collect();
    let before = catalog.clone();
    assert_eq!(upsert_custom(&mut catalog, custom("dQw4w9WgXcQ")), Err(CustomCodesExhausted));
    assert_eq!(catalog, before, "a failed add must not change the catalog");

    // An existing video keeps its code, so re-adding it still succeeds
    let existing = catalog[5].clone();
    let renamed = app::types::Song { title: "Renamed".to_string(), ..existing.clone() };
    assert_eq!(upsert_custom(&mut catalog, renamed).map(|s| s.code), Ok(existing.code));
}
