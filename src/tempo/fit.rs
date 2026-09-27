//! Taps → beat grid (least squares, tolerant of a missed or doubled tap), and the count-in into a part.

use super::types::{BeatGrid, BeatGrids, CountIn};

/// Taps needed for a grid (three intervals).
pub const MIN_TAPS: usize = 4;
/// A longer pause between taps starts a new series (the musician is tapping again from scratch).
pub const TAP_GAP_SECS: f64 = 2.0;
pub const MIN_BPM: f64 = 40.0;
pub const MAX_BPM: f64 = 240.0;
/// Clicks before the part.
pub const COUNT_IN_BEATS: u32 = 4;
/// Playback starts this long before the first click, so the video is already running when it sounds.
pub const PRE_ROLL_SECS: f64 = 1.0;
/// A tap further than this from its beat (as a share of a beat) is a slip and is left out of the fit.
const MAX_TAP_ERROR: f64 = 0.2;

impl BeatGrid {
    pub fn bpm(&self) -> f64 {
        60.0 / self.period
    }

    /// The beat closest to `t`.
    pub fn nearest_beat(&self, t: f64) -> f64 {
        self.anchor + ((t - self.anchor) / self.period).round() * self.period
    }

    /// Count in to the beat nearest `part_start`: `COUNT_IN_BEATS` clicks on the beats before it. Clicks that would
    /// fall before the song's start are left out (a part in the first bar gets a shorter count-in).
    pub fn count_in(&self, part_start: f64) -> CountIn {
        let land = self.nearest_beat(part_start);
        let clicks: Vec<f64> =
            (1..=COUNT_IN_BEATS).rev().map(|k| land - f64::from(k) * self.period).filter(|&t| t >= 0.0).collect();
        let first = clicks.first().copied().unwrap_or(land);
        CountIn { start: (first - PRE_ROLL_SECS).max(0.0), clicks, land }
    }

    fn is_valid(&self) -> bool {
        self.period.is_finite() && self.anchor.is_finite() && (MIN_BPM..=MAX_BPM).contains(&self.bpm())
    }
}

/// The grid a series of taps (karaoke seconds, in order) describes, or `None` for too few or no steady beat.
/// Each tap gets the beat number the median interval suggests, so a skipped beat counts as two; a line through
/// (beat number, time) gives period and anchor. Taps far from the median grid are left out of that line.
pub fn fit(taps: &[f64]) -> Option<BeatGrid> {
    if taps.len() < MIN_TAPS {
        return None;
    }
    let mut intervals: Vec<f64> = taps.windows(2).map(|w| w[1] - w[0]).filter(|d| *d > 0.0).collect();
    intervals.sort_by(f64::total_cmp);
    let median = *intervals.get(intervals.len() / 2)?;
    let beats: Vec<(f64, f64)> = taps.iter().map(|&t| (((t - taps[0]) / median).round(), t)).collect();
    // Slips are judged against a median grid (a least-squares line would lean toward them), then refitted without
    let mut offsets: Vec<f64> = beats.iter().map(|&(k, t)| t - k * median).collect();
    offsets.sort_by(f64::total_cmp);
    let rough = BeatGrid { period: median, anchor: offsets[offsets.len() / 2] };
    let kept: Vec<(f64, f64)> = beats
        .into_iter()
        .filter(|&(k, t)| ((t - (rough.anchor + k * rough.period)) / rough.period).abs() <= MAX_TAP_ERROR)
        .collect();
    if kept.len() < MIN_TAPS {
        return None;
    }
    let grid = line(&kept)?;
    grid.is_valid().then_some(grid)
}

/// Least-squares `t = anchor + k * period`.
fn line(points: &[(f64, f64)]) -> Option<BeatGrid> {
    let n = points.len() as f64;
    let (sk, st) = points.iter().fold((0.0, 0.0), |(a, b), &(k, t)| (a + k, b + t));
    let (mk, mt) = (sk / n, st / n);
    let (num, den) = points.iter().fold((0.0, 0.0), |(num, den), &(k, t)| (num + (k - mk) * (t - mt), den + (k - mk).powi(2)));
    if den == 0.0 {
        return None;
    }
    let period = num / den;
    (period > 0.0).then_some(BeatGrid { period, anchor: mt - period * mk })
}

/// Keep the new series' taps: a pause longer than [`TAP_GAP_SECS`] (or a tap earlier than the last, after a seek)
/// starts over.
pub fn add_tap(taps: &mut Vec<f64>, t: f64) {
    if taps.last().is_some_and(|&last| t - last > TAP_GAP_SECS || t <= last) {
        taps.clear();
    }
    taps.push(t);
}

/// Saved grids with anything unusable dropped (a hand-edited or damaged save).
pub fn sanitize(grids: BeatGrids) -> BeatGrids {
    grids.into_iter().filter(|(_, g)| g.is_valid()).collect()
}
