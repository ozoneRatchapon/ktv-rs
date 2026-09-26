use serde::Deserialize;

use crate::search::search_key;
use crate::types::Song;

/// `assets/library.json`, written by `tools/harvest_library.py`: every official karaoke upload of each label.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LibraryFile {
    pub channels: Vec<LibraryChannel>,
}

/// One label's channel; its songs share the channel name and intro skip.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LibraryChannel {
    pub name: String,
    pub intro_skip_secs: u32,
    pub songs: Vec<LibraryRow>,
}

/// A song as a compact JSON array: `[youtube_id, code, duration_secs, title, artist, alias]` (alias may be "").
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LibraryRow(pub String, pub u32, pub u32, pub String, pub String, pub String);

/// Library songs with their search keys, worked out once: normalising 8000 titles per keystroke took ~20 ms.
#[derive(Debug)]
pub struct Songbook {
    songs: Vec<Song>,
    keys: Vec<String>,
}

impl Songbook {
    pub fn new(songs: Vec<Song>) -> Self {
        let keys = songs.iter().map(search_key).collect();
        Self { songs, keys }
    }
}

/// The loaded library (or none yet), compared by identity: it is set once, so props holding it never diff 8000 songs.
#[derive(Debug, Clone, Copy, Default)]
pub struct Library(Option<&'static Songbook>);

impl Library {
    pub fn new(book: &'static Songbook) -> Self {
        Self(Some(book))
    }

    pub fn songs(self) -> &'static [Song] {
        self.0.map_or(&[], |book| book.songs.as_slice())
    }

    /// The precomputed [`search_key`] when `song` is one of this library's own entries (not a copy).
    /// O(1): the entry's index follows from its address in the library's slice.
    pub fn search_key(self, song: &Song) -> Option<&'static str> {
        let book = self.0?;
        let (start, song) = (book.songs.as_ptr() as usize, song as *const Song as usize);
        let index = song.checked_sub(start)? / std::mem::size_of::<Song>();
        (index < book.songs.len() && start + index * std::mem::size_of::<Song>() == song).then(|| book.keys[index].as_str())
    }
}

impl PartialEq for Library {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            (a, b) => a.is_none() && b.is_none(),
        }
    }
}
