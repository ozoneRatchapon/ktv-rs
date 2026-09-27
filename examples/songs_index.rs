//! Song index for the phone request page (`/songs.txt`, format in `app::library::song_index`): the curated
//! catalog, then the full library, with the booth's own codes. Run by tools/build_web.sh:
//! `cargo run --quiet --example songs_index > dist/songs.txt`.

use std::io::{self, Write};

use app::catalog::builtin_catalog;
use app::library::{parse, song_index, Library, Songbook};

fn main() -> io::Result<()> {
    let library = parse(include_str!("../assets/library.json")).map_err(io::Error::other)?;
    // Library wants the songbook for the whole run (the app keeps it in a static)
    let lib = Library::new(Box::leak(Box::new(Songbook::new(library))));
    io::stdout().lock().write_all(song_index(builtin_catalog().iter().chain(lib.songs())).as_bytes())
}
