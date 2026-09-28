//! Medley (plan 004): parts of several songs, marked by the host or guessed, sung back to back as one queue entry.

mod plan;
mod share;
mod types;

pub use plan::{
    display_title, guess_span, nudge_span, shown_title, slots, GUESS_INTRO_SECS, GUESS_LEN_SECS, MAX_PARTS, MAX_SAVED, MAX_TITLE_CHARS,
    MIN_PARTS, MIN_PART_SECS,
};
pub use share::{parse_fragment, share_fragment, Opened, SharedMedley, SharedPart};
pub use types::{Edge, Medley, MedleyBook, MedleyError, MedleyPart, MedleySlot, PartSource, Span};
