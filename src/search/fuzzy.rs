//! Typo tolerance: the fewest edits (insert, delete, change one character) that turn the query into some part
//! of a song's search key. Used only when nothing matches as typed or on the other keyboard layout.

/// Edits allowed for a normalised query of `len` characters: short queries must match as typed (one edit would
/// match almost anything), then one typo, then two.
pub fn allowed_edits(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

/// Fewest edits to turn `needle` into a substring of `hay` (Sellers' approximate substring match), or `None`
/// when more than `max` are needed. `row` is scratch space reused across calls.
pub fn substring_edits(needle: &[char], hay: &str, max: usize, row: &mut Vec<usize>) -> Option<usize> {
    let m = needle.len();
    row.clear();
    row.extend(0..=m);
    let mut best = row[m];
    for h in hay.chars() {
        // A match may start anywhere in `hay`: the empty prefix costs nothing
        let mut diagonal = 0;
        row[0] = 0;
        for i in 1..=m {
            let above = row[i];
            let cost = usize::from(needle[i - 1] != h);
            row[i] = (diagonal + cost).min(above + 1).min(row[i - 1] + 1);
            diagonal = above;
        }
        best = best.min(row[m]);
        if best == 0 {
            break;
        }
    }
    (best <= max).then_some(best)
}
