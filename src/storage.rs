//! `localStorage` persistence for settings and the session (current song, queue, custom songs).
//! Keys are versioned: a schema change bumps the key, and an unreadable value falls back to defaults
//! instead of breaking startup. Codec and reconciliation are pure, so they are tested on the host.

use std::collections::HashSet;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::catalog::next_custom_code;
use crate::types::{QueueItem, Song};

pub const SETTINGS_KEY: &str = "ktv.settings.v1";
pub const SESSION_KEY: &str = "ktv.session.v1";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub current: Option<QueueItem>,
    pub queue: Vec<QueueItem>,
    pub next_queue_id: u64,
    /// Songs added via "Add URL" (not in `assets/catalog.json`).
    pub custom_songs: Vec<Song>,
}

impl Session {
    /// Snapshot the live state; only songs missing from the built-in catalog are kept as custom (deduped by id).
    pub fn capture(
        current: Option<QueueItem>,
        queue: Vec<QueueItem>,
        next_queue_id: u64,
        catalog: &[Song],
        builtin: &[Song],
    ) -> Self {
        let mut seen: HashSet<&str> = builtin.iter().map(|s| s.id.as_str()).collect();
        let custom_songs = catalog.iter().filter(|s| seen.insert(s.id.as_str())).cloned().collect();
        Self { current, queue, next_queue_id, custom_songs }
    }

    /// Make a stored session safe to resume against the current build:
    /// catalog songs take their latest data (e.g. a re-measured guide), and queue ids never collide.
    pub fn reconcile(mut self, builtin: &[Song]) -> Self {
        let refresh = |item: &mut QueueItem| {
            if let Some(fresh) = builtin.iter().find(|s| s.id == item.song.id) {
                item.song = fresh.clone();
            }
        };
        self.current.iter_mut().for_each(refresh);
        self.queue.iter_mut().for_each(refresh);
        let max_id = self.current.iter().chain(&self.queue).map(|it| it.queue_id).max().unwrap_or(0);
        self.next_queue_id = self.next_queue_id.max(max_id + 1);
        let builtin_ids: HashSet<&str> = builtin.iter().map(|s| s.id.as_str()).collect();
        self.custom_songs.retain(|s| !builtin_ids.contains(s.id.as_str()));
        self.renumber_colliding_codes(builtin);
        self
    }

    /// Older sessions gave every custom song code "99999"; give any code that is taken
    /// (by a built-in or an earlier custom song) a free reserved one, in queue items too.
    fn renumber_colliding_codes(&mut self, builtin: &[Song]) {
        let mut known = builtin.to_vec();
        for song in &mut self.custom_songs {
            if known.iter().any(|s| s.code == song.code) {
                match next_custom_code(&known) {
                    Some(code) => song.code = code,
                    None => return,
                }
            }
            known.push(song.clone());
        }
        let custom = &self.custom_songs;
        let relabel = |item: &mut QueueItem| {
            if let Some(fresh) = custom.iter().find(|s| s.id == item.song.id) {
                item.song.code = fresh.code.clone();
            }
        };
        self.current.iter_mut().for_each(relabel);
        self.queue.iter_mut().for_each(relabel);
    }
}

pub fn decode<T: DeserializeOwned>(raw: Option<&str>) -> Option<T> {
    serde_json::from_str(raw?).ok()
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// `None` when storage is unavailable (private mode, disabled, non-browser) or the value is unreadable.
pub fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    #[cfg(target_arch = "wasm32")]
    {
        decode(storage()?.get_item(key).ok()?.as_deref())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        None
    }
}

/// Best effort: a full or disabled storage must never break playback.
pub fn save<T: Serialize>(key: &str, value: &T) {
    #[cfg(target_arch = "wasm32")]
    if let (Some(store), Ok(json)) = (storage(), serde_json::to_string(value)) {
        let _ = store.set_item(key, &json);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (key, value);
}
