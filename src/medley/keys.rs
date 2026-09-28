//! The key hint (plan 004 M4): each part's key from its song's chord chart on this device, and an order whose
//! joins move less on the circle of fifths. A hint only; nothing is transposed.

use crate::chords::{order_cost, smoothest_order, ChordCharts, Key};

use super::types::Medley;

/// Each part's key, from the chords sounding inside the part (None: no chart for the song, or too few chords).
pub fn part_keys(medley: &Medley, charts: &ChordCharts) -> Vec<Option<Key>> {
    medley.parts.iter().map(|p| charts.get(&p.song_id).and_then(|c| c.key_between(p.span.start, p.span.end))).collect()
}

/// A smoother order (indexes into the parts), when every part's key is known and the order beats the current one.
pub fn key_order(keys: &[Option<Key>]) -> Option<Vec<usize>> {
    let keys = keys.iter().copied().collect::<Option<Vec<Key>>>()?;
    let order = smoothest_order(&keys);
    let reordered = order.iter().map(|&i| keys[i]).collect::<Vec<_>>();
    (order_cost(&reordered) < order_cost(&keys)).then_some(order)
}
