use super::types::{Picks, Shelf};
use crate::library::Library;
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

    /// Songs on a shelf: catalog then library order, except Recent which is newest first.
    /// Unknown ids (e.g. a library song before the library has loaded) are skipped.
    pub fn shelf<'a>(&self, catalog: &'a [Song], library: Library, shelf: &Shelf) -> Vec<&'a Song> {
        let songs = catalog.iter().chain(library.songs());
        match shelf {
            Shelf::All => songs.collect(),
            Shelf::Category(name) => songs.filter(|s| s.category == *name).collect(),
            Shelf::Favourites => songs.filter(|s| self.is_favourite(&s.id)).collect(),
            Shelf::Recent => self.recent.iter().filter_map(|id| songs.clone().find(|s| &s.id == id)).collect(),
        }
    }
}
