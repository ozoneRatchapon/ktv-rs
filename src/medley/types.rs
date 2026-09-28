//! A medley: parts of several songs sung as one queue entry (plan 004).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A stretch of a karaoke video, in karaoke seconds (`end > start`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: f64,
    pub end: f64,
}

/// Where a part's times came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartSource {
    /// The host marked it with A / B while the song played.
    Marked,
    /// Guessed from the song's length (no audio is analysed), so often off: the builder says so.
    Guessed,
}

/// One song's part in a medley.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MedleyPart {
    pub song_id: String,
    pub code: String,
    pub title: String,
    pub artist: String,
    /// The song's length, which a nudged part must stay within (0 = unknown).
    pub duration_secs: u32,
    pub span: Span,
    pub source: PartSource,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Medley {
    pub title: String,
    pub parts: Vec<MedleyPart>,
}

/// Everything medley on this device (`ktv.medleys.v1`): the one being built, saved ones, and the parts the host
/// marked per song (a marked part wins over a guess the next time that song is added).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MedleyBook {
    pub draft: Medley,
    pub saved: Vec<Medley>,
    pub marked: HashMap<String, Span>,
}

/// A queue entry that is one part of a medley: where it sits in the medley and what to play.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MedleySlot {
    pub title: String,
    /// 0-based position in the medley.
    pub index: u32,
    pub count: u32,
    pub span: Span,
}

/// Which end of a part a nudge moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Start,
    End,
}

/// Why a medley cannot be saved or queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MedleyError {
    /// A medley needs at least [`super::MIN_PARTS`] parts.
    TooFewParts,
    /// The draft already holds [`super::MAX_PARTS`] parts.
    TooManyParts,
    /// [`super::MAX_SAVED`] medleys are saved already.
    BookFull,
    /// A marked part is shorter than [`super::MIN_PART_SECS`] or runs past the song's end.
    BadPart,
}

impl MedleyError {
    pub fn message(self) -> String {
        match self {
            Self::TooFewParts => format!("A medley needs at least {} parts", super::MIN_PARTS),
            Self::TooManyParts => format!("A medley holds at most {} parts", super::MAX_PARTS),
            Self::BookFull => format!("{} medleys are saved already: delete one first", super::MAX_SAVED),
            Self::BadPart => format!("A part must be at least {} s long and inside the song", super::MIN_PART_SECS),
        }
    }
}
