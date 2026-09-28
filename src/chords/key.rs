//! A song's key estimated from the chords a musician entered (no audio is analysed), and how far apart two keys
//! are on the circle of fifths: the medley builder's key hint (plan 004 M4).

use std::fmt;

use super::types::{Chord, ChordChart};

/// Fewer chord changes than this in a stretch say too little about its key.
pub const MIN_KEY_MARKS: usize = 3;
/// [`smoothest_order`] leaves longer lists as they are (its work doubles with each key).
const MAX_ORDER_KEYS: usize = 16;

/// Krumhansl–Kessler key profiles (probe-tone ratings), index 0 = the tonic.
const MAJOR_PROFILE: [f64; 12] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
const MINOR_PROFILE: [f64; 12] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

const MAJOR_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
const MINOR_NAMES: [&str; 12] = ["Cm", "C#m", "Dm", "Ebm", "Em", "Fm", "F#m", "Gm", "G#m", "Am", "Bbm", "Bm"];

/// A key: tonic pitch class (0 = C) and mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub tonic: u8,
    pub minor: bool,
}

/// How a join from one key to the next sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStep {
    Same,
    /// Relative major / minor, or one step on the circle of fifths.
    Near,
    Far,
}

/// Semitones above the root of a chord quality's tones (canonical spellings, see `symbol::QUALITIES`).
fn tones(quality: &str) -> &'static [u8] {
    match quality {
        "m" => &[0, 3, 7],
        "7" => &[0, 4, 7, 10],
        "maj7" => &[0, 4, 7, 11],
        "m7" => &[0, 3, 7, 10],
        "6" => &[0, 4, 7, 9],
        "m6" => &[0, 3, 7, 9],
        "9" => &[0, 4, 7, 10, 2],
        "maj9" => &[0, 4, 7, 11, 2],
        "m9" => &[0, 3, 7, 10, 2],
        "11" => &[0, 4, 7, 10, 5],
        "m11" => &[0, 3, 7, 10, 5],
        "13" => &[0, 4, 7, 10, 9],
        "m13" => &[0, 3, 7, 10, 9],
        "maj13" => &[0, 4, 7, 11, 9],
        "add9" => &[0, 4, 7, 2],
        "madd9" => &[0, 3, 7, 2],
        "mmaj7" => &[0, 3, 7, 11],
        "sus4" => &[0, 5, 7],
        "sus2" => &[0, 2, 7],
        "7sus4" => &[0, 5, 7, 10],
        "7sus2" => &[0, 2, 7, 10],
        "dim" => &[0, 3, 6],
        "dim7" => &[0, 3, 6, 9],
        "m7b5" => &[0, 3, 6, 10],
        "aug" => &[0, 4, 8],
        "5" => &[0, 7],
        _ => &[0, 4, 7],
    }
}

/// Pearson correlation of a pitch-class weighting with a key profile rotated to `tonic`.
fn correlation(weights: &[f64; 12], profile: &[f64; 12], tonic: usize) -> f64 {
    let mean_w = weights.iter().sum::<f64>() / 12.0;
    let mean_p = profile.iter().sum::<f64>() / 12.0;
    let (mut num, mut var_w, mut var_p) = (0.0, 0.0, 0.0);
    for (pc, w) in weights.iter().enumerate() {
        let (dw, dp) = (w - mean_w, profile[(pc + 12 - tonic) % 12] - mean_p);
        num += dw * dp;
        var_w += dw * dw;
        var_p += dp * dp;
    }
    let den = (var_w * var_p).sqrt();
    if den > 0.0 { num / den } else { 0.0 }
}

impl Key {
    /// The key whose profile best fits pitch-class weights (None when all weights are equal, e.g. none).
    pub fn best_fit(weights: &[f64; 12]) -> Option<Self> {
        let mut best: Option<(f64, Self)> = None;
        for tonic in 0..12u8 {
            for (minor, profile) in [(false, &MAJOR_PROFILE), (true, &MINOR_PROFILE)] {
                let r = correlation(weights, profile, usize::from(tonic));
                if best.is_none_or(|(top, _)| r > top) {
                    best = Some((r, Self { tonic, minor }));
                }
            }
        }
        best.filter(|(r, _)| *r > 0.0).map(|(_, key)| key)
    }

    /// Position on the circle of fifths (C = 0, G = 1, ...); a minor key sits with its relative major.
    fn fifths(self) -> i32 {
        let major = if self.minor { (self.tonic + 3) % 12 } else { self.tonic };
        i32::from(major) * 7 % 12
    }

    /// Steps between the two keys' places on the circle of fifths (0 to 6).
    pub fn fifths_apart(self, other: Self) -> u32 {
        let d = (self.fifths() - other.fifths()).rem_euclid(12);
        d.min(12 - d) as u32
    }

    /// How moving from this key to `next` sounds.
    pub fn step(self, next: Self) -> KeyStep {
        match (self == next, self.fifths_apart(next)) {
            (true, _) => KeyStep::Same,
            (false, 0 | 1) => KeyStep::Near,
            _ => KeyStep::Far,
        }
    }

    /// Cost of a join for ordering: fifths steps count double, a change of mode adds one.
    pub fn distance(self, next: Self) -> u32 {
        self.fifths_apart(next) * 2 + u32::from(self.minor != next.minor)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = if self.minor { &MINOR_NAMES } else { &MAJOR_NAMES };
        f.write_str(names[usize::from(self.tonic % 12)])
    }
}

impl KeyStep {
    pub fn label(self) -> &'static str {
        match self {
            Self::Same => "same key",
            Self::Near => "near key",
            Self::Far => "far key",
        }
    }
}

impl ChordChart {
    /// The key of the stretch `start..end` (karaoke seconds), from the chords sounding in it weighted by how long
    /// each sounds (roots and basses count double). None with fewer than [`MIN_KEY_MARKS`] chords sounding in it.
    pub fn key_between(&self, start: f64, end: f64) -> Option<Key> {
        let marks = self.marks();
        let first = marks.partition_point(|m| m.at_secs <= start).saturating_sub(1);
        let mut weights = [0.0; 12];
        let mut sounding = 0;
        for (i, mark) in marks.iter().enumerate().skip(first) {
            if mark.at_secs >= end {
                break;
            }
            let until = marks.get(i + 1).map_or(end, |next| next.at_secs.min(end));
            let secs = until - mark.at_secs.max(start);
            let (true, Ok(chord)) = (secs > 0.0, mark.chord.parse::<Chord>()) else { continue };
            sounding += 1;
            for &tone in tones(chord.quality) {
                weights[usize::from((chord.root + tone) % 12)] += secs;
            }
            weights[usize::from(chord.root)] += secs;
            if let Some(bass) = chord.bass {
                weights[usize::from(bass)] += secs;
            }
        }
        (sounding >= MIN_KEY_MARKS).then(|| Key::best_fit(&weights)).flatten()
    }
}

/// The order of `keys` with the least total [`Key::distance`] over its joins (any part may open). Exact
/// (Held–Karp over subsets), meant for a medley's handful of parts; of a path's two directions, the one that
/// keeps earlier parts earlier.
pub fn smoothest_order(keys: &[Key]) -> Vec<usize> {
    let n = keys.len();
    if !(3..=MAX_ORDER_KEYS).contains(&n) {
        return (0..n).collect();
    }
    let full = (1usize << n) - 1;
    // cost[mask * n + last]: least cost of a path over `mask` ending at `last`
    let mut cost = vec![u32::MAX; (full + 1) * n];
    let mut prev = vec![usize::MAX; (full + 1) * n];
    for i in 0..n {
        cost[(1 << i) * n + i] = 0;
    }
    for mask in 1..=full {
        for last in 0..n {
            let here = cost[mask * n + last];
            if here == u32::MAX {
                continue;
            }
            for next in (0..n).filter(|&j| mask & (1 << j) == 0) {
                let (to, c) = ((mask | (1 << next)) * n + next, here + keys[last].distance(keys[next]));
                if c < cost[to] {
                    cost[to] = c;
                    prev[to] = last;
                }
            }
        }
    }
    let mut last = (0..n).min_by_key(|&i| cost[full * n + i]).unwrap_or(0);
    let (mut mask, mut order) = (full, Vec::with_capacity(n));
    while last != usize::MAX {
        order.push(last);
        let before = prev[mask * n + last];
        mask &= !(1 << last);
        last = before;
    }
    // A path costs the same both ways: take the way that keeps earlier parts earlier (the host's opener first)
    let reversed = order.clone();
    order.reverse();
    order.min(reversed)
}

/// Total [`Key::distance`] of the joins in this order.
pub fn order_cost(keys: &[Key]) -> u32 {
    keys.windows(2).map(|pair| pair[0].distance(pair[1])).sum()
}
