use super::types::{Picks, Shelf};
use crate::types::Song;

/// Recently sung songs kept (older ones drop off).
pub const MAX_RECENT: usize = 20;
/// A song counts as sung after this long on stage (a quick skip does not).
pub const MIN_SUNG_SECS: f64 = 30.0;

impl Picks {
    pub fn is_favourite(&self, song_id: &str) -> bool {
        self.favourites.iter().any(|id| id == song_id)
    }

    pub fn toggle_favourite(&mut self, song_id: &str) {
        match self.favourites.iter().position(|id| id == song_id) {
            Some(i) => {
                self.favourites.remove(i);
            }
            None => self.favourites.push(song_id.to_string()),
        }
    }

    /// Note a song left the stage after `secs`; only real takes (≥ [`MIN_SUNG_SECS`]) count.
    pub fn record_sung(&mut self, song_id: &str, secs: f64) {
        if secs < MIN_SUNG_SECS {
            return;
        }
        self.recent.retain(|id| id != song_id);
        self.recent.insert(0, song_id.to_string());
        self.recent.truncate(MAX_RECENT);
    }

    /// Songs on a shelf: catalog order, except Recent which is newest first. Unknown ids are skipped.
    pub fn shelf(&self, catalog: &[Song], shelf: &Shelf) -> Vec<Song> {
        let by_id = |id: &String| catalog.iter().find(|s| &s.id == id).cloned();
        match shelf {
            Shelf::All => catalog.to_vec(),
            Shelf::Category(name) => catalog.iter().filter(|s| s.category == *name).cloned().collect(),
            Shelf::Favourites => catalog.iter().filter(|s| self.is_favourite(&s.id)).cloned().collect(),
            Shelf::Recent => self.recent.iter().filter_map(by_id).collect(),
        }
    }
}
