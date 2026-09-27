use std::fmt;
use std::str::FromStr;

use super::types::{Chord, ChordParseError};

const SHARP_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLAT_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

/// Accepted quality spellings -> canonical. Longest first, so `maj7` is not read as `m` + `aj7`.
const QUALITIES: [(&str, &str); 33] = [
    ("maj13", "maj13"),
    ("maj9", "maj9"),
    ("maj7", "maj7"),
    ("madd9", "madd9"),
    ("m7b5", "m7b5"),
    ("mmaj7", "mmaj7"),
    ("7sus4", "7sus4"),
    ("7sus2", "7sus2"),
    ("add9", "add9"),
    ("add2", "add9"),
    ("sus4", "sus4"),
    ("sus2", "sus2"),
    ("dim7", "dim7"),
    ("min7", "m7"),
    ("min", "m"),
    ("aug", "aug"),
    ("dim", "dim"),
    ("sus", "sus4"),
    ("m13", "m13"),
    ("m11", "m11"),
    ("m9", "m9"),
    ("m7", "m7"),
    ("m6", "m6"),
    ("M7", "maj7"),
    ("13", "13"),
    ("11", "11"),
    ("°7", "dim7"),
    ("m", "m"),
    ("9", "9"),
    ("7", "7"),
    ("6", "6"),
    ("5", "5"),
    ("°", "dim"),
];

/// `(pitch class, spelled with a flat)` of a note name at the start of `text`, and the bytes it used.
fn parse_note(text: &str) -> Option<(u8, bool, usize)> {
    let mut chars = text.chars();
    let base = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    // The letter is ASCII (1 byte); `♯` / `♭` are 3
    match chars.next() {
        Some(c @ ('#' | '♯')) => Some(((base + 1) % 12, false, 1 + c.len_utf8())),
        Some(c @ ('b' | '♭')) => Some(((base + 11) % 12, true, 1 + c.len_utf8())),
        _ => Some((base, false, 1)),
    }
}

impl FromStr for Chord {
    type Err = ChordParseError;

    /// `C`, `Am`, `F#m7`, `Bbmaj7`, `Dsus4`, `G/B`, `C♯m`, `Hm` is rejected (German naming).
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ChordParseError::Empty);
        }
        let (root, flats, used) = parse_note(text).ok_or(ChordParseError::Root)?;
        let rest = &text[used..];
        let (quality_text, bass_text) = match rest.split_once('/') {
            Some((q, b)) => (q, Some(b)),
            None => (rest, None),
        };
        let quality = match quality_text {
            "" => "",
            "+" => "aug",
            q => QUALITIES.iter().find(|(spelling, _)| *spelling == q).map(|(_, c)| *c).ok_or(ChordParseError::Quality)?,
        };
        let bass = match bass_text {
            None => None,
            Some(b) => match parse_note(b) {
                Some((pc, _, n)) if n == b.len() => Some(pc),
                _ => return Err(ChordParseError::Bass),
            },
        };
        Ok(Self { root, quality, bass, flats })
    }
}

impl Chord {
    /// The same chord `semitones` higher (negative: lower), keeping the flat or sharp spelling.
    pub fn transposed(self, semitones: i32) -> Self {
        let shift = |pc: u8| (i32::from(pc) + semitones).rem_euclid(12) as u8;
        Self { root: shift(self.root), bass: self.bass.map(shift), ..self }
    }

    fn note_name(self, pc: u8) -> &'static str {
        let names = if self.flats { &FLAT_NAMES } else { &SHARP_NAMES };
        names[usize::from(pc % 12)]
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.note_name(self.root), self.quality)?;
        match self.bass {
            Some(bass) => write!(f, "/{}", self.note_name(bass)),
            None => Ok(()),
        }
    }
}

impl fmt::Display for ChordParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "Type a chord name first",
            Self::Root => "A chord starts with a note, A to G (e.g. Am, F#m7, Bb)",
            Self::Quality => "Unknown chord type: try m, 7, m7, maj7, sus4, dim, aug, add9",
            Self::Bass => "After / comes the bass note, A to G (e.g. G/B)",
        })
    }
}

impl std::error::Error for ChordParseError {}
