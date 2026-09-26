//! The singer's own shelves on this device: favourites and recently sung songs.

mod ops;
mod types;

pub use ops::{MAX_RECENT, MIN_SUNG_SECS};
pub use types::{Picks, Shelf};
