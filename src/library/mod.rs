//! The full songbook: every official karaoke upload from the labels' channels (`assets/library.json`).
//! Too big to compile into the wasm (≈300 KB gzip), so it is fetched after the first paint and set once;
//! until then only the curated `assets/catalog.json` songs are listed.

mod load;
mod mv;
mod types;

pub use load::{install, loaded, parse, ID_PREFIX};
pub use mv::{auto_timed, guides, install_guides, suggested_video, timed_guide, MvGuide};
pub use types::{Library, LibraryChannel, LibraryFile, LibraryRow, Songbook};
