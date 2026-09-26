//! Values captured from the real app (release build, 2026-09-27) for every `localStorage` key. A change to the
//! stored types must keep decoding these, or bump the key and migrate: a decode failure silently resets the
//! user's data to defaults (see `storage::load`).

use app::catalog::builtin_catalog;
use app::picks::Picks;
use app::score::TakeResult;
use app::storage::{decode, Session, GUIDES_KEY, PICKS_KEY, SCORES_KEY, SESSION_KEY, SETTINGS_KEY};
use app::timing::GuideOverrides;
use app::types::AppSettings;

const SETTINGS: &str = include_str!("fixtures/storage/settings_v1.json");
const SESSION: &str = include_str!("fixtures/storage/session_v1.json");
const GUIDES: &str = include_str!("fixtures/storage/guides_v1.json");
const SCORES: &str = include_str!("fixtures/storage/scores_v1.json");
const PICKS: &str = include_str!("fixtures/storage/picks_v1.json");

#[test]
fn test_fixture_names_match_the_storage_keys() {
    assert_eq!(
        [SETTINGS_KEY, SESSION_KEY, GUIDES_KEY, SCORES_KEY, PICKS_KEY],
        ["ktv.settings.v1", "ktv.session.v1", "ktv.guides.v1", "ktv.scores.v1", "ktv.picks.v1"],
        "a key changed: add a fixture for the new version and keep this one for the migration"
    );
}

#[test]
fn test_saved_settings_still_decode() {
    let settings = decode::<AppSettings>(Some(SETTINGS)).expect("settings v1");
    assert_eq!(settings.room_name, "VIP ROOM 07");
    assert!(settings.show_timing_tools);
}

#[test]
fn test_saved_session_still_decodes_and_reconciles() {
    let session = decode::<Session>(Some(SESSION)).expect("session v1");
    assert_eq!(session.custom_songs.len(), 1);
    let custom_id = session.custom_songs[0].id.clone();
    let session = session.reconcile(builtin_catalog());
    assert!(session.current.is_some());
    assert!(session.queue.iter().any(|it| it.song.id == custom_id), "Add URL song survives in the queue");
    assert_eq!(session.custom_songs.len(), 1);
}

#[test]
fn test_saved_guides_scores_and_picks_still_decode() {
    let guides = decode::<GuideOverrides>(Some(GUIDES)).expect("guides v1");
    assert!((guides["gmm_004"].rate - 1.0012).abs() < 1e-9);
    let scores = decode::<Vec<TakeResult>>(Some(SCORES)).expect("scores v1");
    assert_eq!(scores[0].song_id, "gmm_004");
    let picks = decode::<Picks>(Some(PICKS)).expect("picks v1");
    assert_eq!(picks.favourites, ["gmm_005"]);
}
