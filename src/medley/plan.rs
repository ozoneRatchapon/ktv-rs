//! Medley building: parts (marked or guessed), the draft, saved medleys, and the queue slots a medley becomes.

use super::types::{Edge, Medley, MedleyBook, MedleyError, MedleyPart, MedleySlot, PartSource, Span};
use crate::types::Song;

/// A part shorter than this is a slip of the finger, not a section.
pub const MIN_PART_SECS: f64 = 5.0;
pub const MIN_PARTS: usize = 2;
pub const MAX_PARTS: usize = 12;
pub const MAX_SAVED: usize = 50;
/// Characters kept of a medley's title.
pub const MAX_TITLE_CHARS: usize = 60;
/// A guessed part starts after the song's intro skip, or this far in when the channel has none (title card and
/// intro music), and runs long enough for a typical first verse and chorus.
pub const GUESS_INTRO_SECS: f64 = 15.0;
pub const GUESS_LEN_SECS: f64 = 75.0;

impl Span {
    /// A usable part of a song `duration_secs` long (0 = length unknown, so no upper bound).
    pub fn within(start: f64, end: f64, duration_secs: u32) -> Option<Span> {
        let fits = duration_secs == 0 || end <= f64::from(duration_secs);
        (start.is_finite() && end.is_finite() && start >= 0.0 && end - start >= MIN_PART_SECS && fits)
            .then_some(Span { start, end })
    }

    pub fn len(&self) -> f64 {
        self.end - self.start
    }
}

/// The part to try when the host has not marked one: after the intro, about a verse and a chorus long.
/// Nothing listens to the song, so this is a starting point to nudge, not a found chorus.
pub fn guess_span(song: &Song) -> Span {
    let duration = f64::from(song.duration_secs);
    let start = f64::from(song.intro_skip_secs).max(GUESS_INTRO_SECS);
    match song.duration_secs {
        0 => Span { start, end: start + GUESS_LEN_SECS },
        _ if start + MIN_PART_SECS > duration => Span { start: 0.0, end: duration.max(MIN_PART_SECS) },
        _ => Span { start, end: (start + GUESS_LEN_SECS).min(duration) },
    }
}

fn part(song: &Song, span: Span, source: PartSource) -> MedleyPart {
    MedleyPart {
        song_id: song.id.clone(),
        code: song.code.clone(),
        title: song.title.clone(),
        artist: song.artist.clone(),
        duration_secs: song.duration_secs,
        span,
        source,
    }
}

/// `span` moved at one edge by `delta` seconds, kept inside the song and at least [`MIN_PART_SECS`] long.
pub fn nudge_span(span: Span, edge: Edge, delta: f64, duration_secs: u32) -> Span {
    let limit = match duration_secs {
        0 => f64::INFINITY,
        d => f64::from(d),
    };
    match edge {
        Edge::Start => Span { start: (span.start + delta).clamp(0.0, (span.end - MIN_PART_SECS).max(0.0)), ..span },
        Edge::End => Span { end: (span.end + delta).clamp(span.start + MIN_PART_SECS, limit.max(span.start + MIN_PART_SECS)), ..span },
    }
}

/// The medley's title, or a stand-in while it has none.
pub fn display_title(medley: &Medley) -> &str {
    match medley.title.trim() {
        "" => "Medley",
        title => title,
    }
}

fn clean_title(title: &str) -> String {
    title.trim().chars().take(MAX_TITLE_CHARS).collect()
}

/// The queue entries a medley becomes, in order: each part's song id and its slot.
pub fn slots(medley: &Medley) -> Result<Vec<(String, MedleySlot)>, MedleyError> {
    if medley.parts.len() < MIN_PARTS {
        return Err(MedleyError::TooFewParts);
    }
    let title = display_title(medley).to_string();
    let count = medley.parts.len() as u32;
    Ok(medley
        .parts
        .iter()
        .enumerate()
        .map(|(i, p)| (p.song_id.clone(), MedleySlot { title: title.clone(), index: i as u32, count, span: p.span }))
        .collect())
}

impl MedleySlot {
    /// "Medley 2/4 · Title", for the player and the queue.
    pub fn label(&self) -> String {
        format!("Medley {}/{} · {}", self.index + 1, self.count, self.title)
    }

    /// A later part of its medley: it follows the last one straight on (no MC between parts).
    pub fn is_join(&self) -> bool {
        self.index > 0
    }
}

impl MedleyBook {
    fn push_part(&mut self, part: MedleyPart) -> Result<(), MedleyError> {
        if self.draft.parts.len() >= MAX_PARTS {
            return Err(MedleyError::TooManyParts);
        }
        self.draft.parts.push(part);
        Ok(())
    }

    /// Add a song to the draft: the part the host marked for it before, else a guess.
    pub fn add_song(&mut self, song: &Song) -> Result<(), MedleyError> {
        let marked = self.marked.get(&song.id).and_then(|s| Span::within(s.start, s.end, song.duration_secs));
        let part = match marked {
            Some(span) => part(song, span, PartSource::Marked),
            None => part(song, guess_span(song), PartSource::Guessed),
        };
        self.push_part(part)
    }

    /// Add a part marked with A / B; it is remembered for the song even when the draft is full.
    pub fn add_marked(&mut self, song: &Song, start: f64, end: f64) -> Result<(), MedleyError> {
        let span = Span::within(start, end, song.duration_secs).ok_or(MedleyError::BadPart)?;
        self.marked.insert(song.id.clone(), span);
        self.push_part(part(song, span, PartSource::Marked))
    }

    /// Move one edge of a draft part; the result counts as marked by the host and is remembered for the song.
    pub fn nudge(&mut self, index: usize, edge: Edge, delta: f64) {
        let Some(p) = self.draft.parts.get_mut(index) else { return };
        p.span = nudge_span(p.span, edge, delta, p.duration_secs);
        p.source = PartSource::Marked;
        self.marked.insert(p.song_id.clone(), p.span);
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.draft.parts.len() {
            self.draft.parts.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.draft.parts.len() {
            self.draft.parts.swap(index, index + 1);
        }
    }

    pub fn remove_part(&mut self, index: usize) {
        if index < self.draft.parts.len() {
            self.draft.parts.remove(index);
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.draft.title = clean_title(title);
    }

    pub fn clear_draft(&mut self) {
        self.draft = Medley::default();
    }

    /// Save the draft (a saved medley with the same title is replaced: that is how an edit is saved).
    pub fn save_draft(&mut self) -> Result<(), MedleyError> {
        if self.draft.parts.len() < MIN_PARTS {
            return Err(MedleyError::TooFewParts);
        }
        if self.draft.title.trim().is_empty() {
            self.draft.title = format!("Medley {}", self.saved.len() + 1);
        }
        let title = display_title(&self.draft).to_string();
        match self.saved.iter().position(|m| display_title(m) == title) {
            Some(i) => self.saved[i] = self.draft.clone(),
            None if self.saved.len() >= MAX_SAVED => return Err(MedleyError::BookFull),
            None => self.saved.push(self.draft.clone()),
        }
        Ok(())
    }

    /// Open a saved medley in the builder.
    pub fn edit(&mut self, index: usize) {
        if let Some(m) = self.saved.get(index) {
            self.draft = m.clone();
        }
    }

    pub fn delete(&mut self, index: usize) {
        if index < self.saved.len() {
            self.saved.remove(index);
        }
    }

    /// A stored book made safe to use: parts outside their song or too short are dropped, saved medleys left
    /// with too few parts are dropped, and every list is cut to its limit.
    pub fn sanitized(mut self) -> Self {
        let clean = |m: &mut Medley| {
            m.title = clean_title(&m.title);
            m.parts.retain(|p| Span::within(p.span.start, p.span.end, p.duration_secs).is_some());
            m.parts.truncate(MAX_PARTS);
        };
        clean(&mut self.draft);
        self.saved.iter_mut().for_each(clean);
        self.saved.retain(|m| m.parts.len() >= MIN_PARTS);
        self.saved.truncate(MAX_SAVED);
        self.marked.retain(|_, s| Span::within(s.start, s.end, 0).is_some());
        self
    }
}
