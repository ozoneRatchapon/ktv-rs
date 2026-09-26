use std::ops::RangeInclusive;

use super::types::{FitError, GuideOverrides, SyncMark};
use crate::types::{GuideTrack, Song};

/// Shortest karaoke span two marks may cover; ear error (~0.05 s) over 30 s keeps rate within ~0.2 %.
pub const MIN_FIT_SPAN_SECS: f64 = 30.0;
/// Plausible MV speed relative to the karaoke track (YouTube edits are the same master, maybe resampled).
pub const RATE_RANGE: RangeInclusive<f64> = 0.9..=1.1;

fn round_to(v: f64, step: f64) -> f64 {
    (v / step).round() * step
}

/// Where the guide is at `karaoke_secs` under `guide`'s mapping (MV = offset + rate * karaoke).
pub fn mark_at(guide: &GuideTrack, karaoke_secs: f64) -> SyncMark {
    let mv_secs = f64::from(guide.offset_secs) + f64::from(guide.rate) * karaoke_secs;
    SyncMark { karaoke_secs, mv_secs }
}

/// Shift the guide by `delta_secs` (positive: the guide plays later material, i.e. it was behind).
pub fn nudge(guide: &GuideTrack, delta_secs: f64) -> GuideTrack {
    let offset = round_to(f64::from(guide.offset_secs) + delta_secs, 0.01);
    GuideTrack { offset_secs: offset as f32, ..guide.clone() }
}

/// Line through two sync marks: the mapping that keeps both in sync. Order of marks does not matter.
pub fn fit(video_id: &str, a: SyncMark, b: SyncMark) -> Result<GuideTrack, FitError> {
    let (first, last) = match a.karaoke_secs <= b.karaoke_secs {
        true => (a, b),
        false => (b, a),
    };
    let span_secs = last.karaoke_secs - first.karaoke_secs;
    if span_secs < MIN_FIT_SPAN_SECS {
        return Err(FitError::TooClose { span_secs });
    }
    let rate = (last.mv_secs - first.mv_secs) / span_secs;
    if !RATE_RANGE.contains(&rate) {
        return Err(FitError::RateOutOfRange { rate });
    }
    let rate = round_to(rate, 0.0001);
    let offset = round_to(first.mv_secs - rate * first.karaoke_secs, 0.01);
    Ok(GuideTrack { video_id: video_id.to_string(), offset_secs: offset as f32, rate: rate as f32 })
}

/// A fresh guide for a newly chosen MV: same timeline until the curator nudges it.
pub fn new_guide(video_id: &str) -> GuideTrack {
    GuideTrack { video_id: video_id.to_string(), offset_secs: 0.0, rate: 1.0 }
}

/// Put in-app guide timings onto songs (catalog, queue, current); songs without an override keep theirs.
pub fn apply_overrides<'a>(songs: impl IntoIterator<Item = &'a mut Song>, overrides: &GuideOverrides) {
    for song in songs {
        if let Some(guide) = overrides.get(&song.id) {
            song.guide = Some(guide.clone());
        }
    }
}

/// The `"guide"` entry to paste into `assets/catalog.json` for a song.
pub fn catalog_snippet(guide: &GuideTrack) -> String {
    let GuideTrack { video_id, offset_secs, rate } = guide;
    format!("\"guide\": {{\n  \"video_id\": \"{video_id}\",\n  \"offset_secs\": {offset_secs},\n  \"rate\": {rate:?}\n}}")
}
