//! Party leaderboard: each named singer's best take of the night. Local only; names never leave the device.

use super::history::TakeResult;

/// Takes this recent count as tonight's party.
pub const PARTY_WINDOW_MS: f64 = 12.0 * 60.0 * 60.0 * 1000.0;
/// Longest singer name kept (characters).
pub const MAX_NAME_CHARS: usize = 24;

/// One leaderboard line: the singer's best-scoring take in the window and how many scored takes they sang.
#[derive(Debug, Clone, PartialEq)]
pub struct LeaderRow {
    pub singer: String,
    pub best: TakeResult,
    pub takes: u32,
}

/// A typed name trimmed, inner whitespace collapsed and capped at [`MAX_NAME_CHARS`]; `None` when blank.
pub fn clean_name(raw: &str) -> Option<String> {
    let name: String = raw.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(MAX_NAME_CHARS).collect();
    let name = name.trim_end().to_string();
    (!name.is_empty()).then_some(name)
}

/// Put `name` on the take that ended at `sung_at_ms` (`None` clears it). False if no such take is kept.
pub fn name_take(history: &mut [TakeResult], sung_at_ms: f64, part: Option<u8>, name: Option<String>) -> bool {
    match history.iter_mut().find(|t| t.sung_at_ms == sung_at_ms && t.part == part) {
        Some(take) => {
            take.singer = name;
            true
        }
        None => false,
    }
}

/// Names used before, newest first, without repeats (case-insensitive): one-tap choices on the result card.
pub fn recent_singers(history: &[TakeResult], limit: usize) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for name in history.iter().filter_map(|t| t.singer.as_deref()) {
        if names.len() == limit {
            break;
        }
        if !names.iter().any(|n| n.to_lowercase() == name.to_lowercase()) {
            names.push(name.to_string());
        }
    }
    names
}

/// Named, scored takes from the last [`PARTY_WINDOW_MS`] before `now_ms`, best take per singer (names match
/// case-insensitively), highest score first; ties go to more held notes, then to whoever sang it first.
pub fn leaderboard(history: &[TakeResult], now_ms: f64) -> Vec<LeaderRow> {
    let tonight = history.iter().filter(|t| now_ms - t.sung_at_ms <= PARTY_WINDOW_MS && t.score().is_some());
    let mut rows: Vec<LeaderRow> = Vec::new();
    for take in tonight {
        let Some(singer) = take.singer.as_deref() else { continue };
        let key = singer.to_lowercase();
        match rows.iter_mut().find(|r| r.singer.to_lowercase() == key) {
            Some(row) => {
                row.takes += 1;
                if better(take, &row.best) {
                    row.best = take.clone();
                }
            }
            None => rows.push(LeaderRow { singer: singer.to_string(), best: take.clone(), takes: 1 }),
        }
    }
    rows.sort_by(|a, b| {
        if better(&a.best, &b.best) {
            std::cmp::Ordering::Less
        } else if better(&b.best, &a.best) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    rows
}

/// `a` ranks above `b`: higher score, then more held notes, then sung earlier.
fn better(a: &TakeResult, b: &TakeResult) -> bool {
    (a.score(), a.notes, -a.sung_at_ms) > (b.score(), b.notes, -b.sung_at_ms)
}
