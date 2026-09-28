//! A medley shared by link: `#medley=<title>&parts=<code>:<start>-<end>[g],…` in the URL fragment, so it never
//! reaches the server. Parts name songs by their keypad code; the one who opens the link resolves them against
//! their own songbook.

use super::plan::{clean_title, part, MAX_PARTS};
use super::types::{Medley, PartSource, Span};
use crate::links::{percent_decode, percent_encode};
use crate::types::Song;

const TITLE_KEY: &str = "medley";
const PARTS_KEY: &str = "parts";
/// Marks a part that was guessed, not marked by the sharer (so the builder keeps saying "guessed").
const GUESSED_FLAG: char = 'g';

/// One part as the link carries it: a song code and its times.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedPart {
    pub code: String,
    pub span: Span,
    pub source: PartSource,
}

/// A medley read from a link, before its songs are looked up.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedMedley {
    pub title: String,
    pub parts: Vec<SharedPart>,
}

/// A shared medley resolved against this device's songbook.
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    pub medley: Medley,
    /// Parts dropped: their song is not in this songbook, or their times do not fit it.
    pub skipped: usize,
}

/// Seconds to one decimal, without a trailing ".0".
fn secs(value: f64) -> String {
    let text = format!("{:.1}", (value * 10.0).round() / 10.0);
    match text.strip_suffix(".0") {
        Some(whole) => whole.to_string(),
        None => text,
    }
}

/// The URL fragment (with its `#`) that opens `medley` in the builder.
pub fn share_fragment(medley: &Medley) -> String {
    let parts: Vec<String> = medley
        .parts
        .iter()
        .map(|p| {
            let flag = match p.source {
                PartSource::Guessed => GUESSED_FLAG.to_string(),
                PartSource::Marked => String::new(),
            };
            format!("{}:{}-{}{flag}", percent_encode(&p.code), secs(p.span.start), secs(p.span.end))
        })
        .collect();
    format!("#{TITLE_KEY}={}&{PARTS_KEY}={}", percent_encode(medley.title.trim()), parts.join(","))
}

fn parse_part(text: &str) -> Option<SharedPart> {
    let (code, times) = text.split_once(':')?;
    let (times, source) = match times.strip_suffix(GUESSED_FLAG) {
        Some(times) => (times, PartSource::Guessed),
        None => (times, PartSource::Marked),
    };
    let (start, end) = times.split_once('-')?;
    let span = Span::within(start.parse().ok()?, end.parse().ok()?, 0)?;
    let code = percent_decode(code)?;
    (!code.is_empty()).then_some(SharedPart { code, span, source })
}

/// The medley in a URL fragment (with or without its `#`), or `None` when the fragment holds no medley.
/// Unreadable parts are left out; at most [`MAX_PARTS`] are kept.
pub fn parse_fragment(fragment: &str) -> Option<SharedMedley> {
    let (mut title, mut parts) = (None, None);
    for pair in fragment.trim_start_matches('#').split('&') {
        match pair.split_once('=') {
            Some((TITLE_KEY, value)) => title = Some(clean_title(&percent_decode(value)?)),
            Some((PARTS_KEY, value)) => parts = Some(value),
            _ => {}
        }
    }
    let parts: Vec<SharedPart> = parts?.split(',').filter_map(parse_part).take(MAX_PARTS).collect();
    (!parts.is_empty()).then(|| SharedMedley { title: title.unwrap_or_default(), parts })
}

impl SharedMedley {
    /// The medley with each code looked up by `find`; parts whose song is missing or too short for the times
    /// are skipped (and counted).
    pub fn open(&self, find: impl Fn(&str) -> Option<Song>) -> Opened {
        let parts: Vec<_> = self
            .parts
            .iter()
            .filter_map(|p| {
                let song = find(&p.code)?;
                let span = Span::within(p.span.start, p.span.end, song.duration_secs)?;
                Some(part(&song, span, p.source))
            })
            .collect();
        Opened { skipped: self.parts.len() - parts.len(), medley: Medley { title: self.title.clone(), parts } }
    }
}
