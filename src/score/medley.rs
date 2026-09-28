//! A medley's total: its parts' scores brought together after the last part (plan 004 M5b).

use super::history::TakeResult;

/// One voice's score over a whole medley.
#[derive(Debug, Clone, PartialEq)]
pub struct MedleyTotal {
    pub title: String,
    /// Mean of the scored parts' scores.
    pub score: u8,
    /// Parts that got a score (a part sung without the mic, or too few held notes, has none).
    pub scored: u32,
    pub count: u32,
}

/// The medley's total when `last` is the take of its last part, from the takes of the same medley and voice (duet
/// singer) in `history` (newest first; a replayed part counts once, its latest take). `last` may already be in
/// `history`. `None` for a whole song, an earlier part, or when no part got a score.
pub fn medley_total(history: &[TakeResult], last: &TakeResult) -> Option<MedleyTotal> {
    let medley = last.medley.as_ref().filter(|m| m.index + 1 == m.count)?;
    let mut seen = vec![false; medley.count as usize];
    let scores: Vec<u32> = std::iter::once(last)
        .chain(history)
        .filter(|t| t.part == last.part)
        .filter_map(|t| {
            let m = t.medley.as_ref().filter(|m| m.id == medley.id)?;
            let first = seen.get_mut(m.index as usize).map(|s| !std::mem::replace(s, true))?;
            first.then(|| t.score()).flatten().map(u32::from)
        })
        .collect();
    let scored = scores.len() as u32;
    (scored > 0).then(|| MedleyTotal {
        title: medley.title.clone(),
        score: (scores.iter().sum::<u32>() as f32 / scored as f32).round() as u8,
        scored,
        count: medley.count,
    })
}
