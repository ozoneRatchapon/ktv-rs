use app::catalog::builtin_catalog;
use app::library::Library;
use app::picks::{Picks, Shelf, MAX_RECENT, MIN_SUNG_SECS};
use app::storage::decode;

fn ids<'a>(songs: &[&'a app::types::Song]) -> Vec<&'a str> {
    songs.iter().map(|s| s.id.as_str()).collect()
}

#[test]
fn test_toggle_favourite() {
    let mut picks = Picks::default();
    picks.toggle_favourite("gmm_004");
    picks.toggle_favourite("gmm_001");
    assert!(picks.is_favourite("gmm_004"));
    picks.toggle_favourite("gmm_004");
    assert!(!picks.is_favourite("gmm_004"));
    assert_eq!(picks.favourites, ["gmm_001"]);
}

#[test]
fn test_recent_needs_a_real_take_and_moves_repeats_to_the_front() {
    let mut picks = Picks::default();
    picks.record_sung("gmm_001", MIN_SUNG_SECS - 1.0);
    assert!(picks.recent.is_empty(), "a quick skip is not a song sung");
    picks.record_sung("gmm_001", 200.0);
    picks.record_sung("gmm_002", 200.0);
    picks.record_sung("gmm_001", 200.0);
    assert_eq!(picks.recent, ["gmm_001", "gmm_002"]);
    for i in 0..MAX_RECENT + 3 {
        picks.record_sung(&format!("song_{i}"), 60.0);
    }
    assert_eq!(picks.recent.len(), MAX_RECENT);
}

#[test]
fn test_shelves() {
    let cat = builtin_catalog();
    let mut picks = Picks::default();
    picks.toggle_favourite(&cat[5].id);
    picks.toggle_favourite(&cat[1].id);
    picks.record_sung(&cat[3].id, 100.0);
    picks.record_sung("gone_song", 100.0);
    picks.record_sung(&cat[0].id, 100.0);
    assert_eq!(picks.shelf(cat, Library::default(), &Shelf::All).len(), cat.len());
    assert_eq!(ids(&picks.shelf(cat, Library::default(), &Shelf::Favourites)), [cat[1].id.as_str(), cat[5].id.as_str()], "catalog order");
    assert_eq!(ids(&picks.shelf(cat, Library::default(), &Shelf::Recent)), [cat[0].id.as_str(), cat[3].id.as_str()], "newest first, unknown ids skipped");
    assert!(picks.shelf(cat, Library::default(), &Shelf::Category("Rock")).iter().all(|s| s.category == "Rock"));
}

#[test]
fn test_picks_storage_tolerates_missing_fields() {
    assert_eq!(decode::<Picks>(Some(r#"{"favourites":["a"]}"#)).map(|p| p.recent.len()), Some(0));
    assert_eq!(decode::<Picks>(Some("{}")), Some(Picks::default()));
}
