use dioxus::prelude::*;

use crate::components::chords::ChordLane;
use crate::components::tempo::TempoTools;
use crate::links::chords_search_url;
use crate::sync::{self, SyncCommand};
use crate::types::Song;

/// A loop shorter than this is a slip of the finger, not a section.
const MIN_LOOP_SECS: f64 = 1.0;

fn format_mark(secs: f64) -> String {
    let whole = secs.max(0.0) as u64;
    format!("{:02}:{:02}", whole / 60, whole % 60)
}

/// Musician tools under the scrubber: loop a section (A then B) to practise it, look up the song's chords, or
/// enter them by ear ([`ChordLane`]).
#[component]
pub fn Practice(
    song: Song,
    /// The queue entry on stage: a new one starts without a loop (the sync core drops it on load too).
    queue_id: u64,
    /// Current karaoke second, used when the sync core cannot give a finer one.
    fallback_sec: u64,
) -> Element {
    let mut start = use_signal(|| None::<f64>);
    let mut end = use_signal(|| None::<f64>);
    use_effect(use_reactive!(|queue_id| {
        let _ = queue_id;
        start.set(None);
        end.set(None);
    }));
    let now = move || sync::karaoke_time().unwrap_or(fallback_sec as f64);

    let label = match (start(), end()) {
        (Some(a), Some(b)) => format!("Looping {} – {}", format_mark(a), format_mark(b)),
        (Some(a), None) => format!("From {}: press B at the end of the part", format_mark(a)),
        _ => "Loop a part: A at its start, B at its end".to_string(),
    };

    rsx! {
        div { class: "practice-row",
            button {
                class: "ctrl-btn practice-btn",
                title: "Loop start: here",
                onclick: move |_| {
                    start.set(Some(now()));
                    end.set(None);
                    SyncCommand::ClearLoop.run();
                },
                "A"
            }
            button {
                class: if end().is_some() { "ctrl-btn practice-btn guide-active" } else { "ctrl-btn practice-btn" },
                title: "Loop end: here, then play A to B over and over",
                disabled: start().is_none(),
                onclick: move |_| {
                    let Some(a) = start() else { return };
                    let b = now();
                    if b - a < MIN_LOOP_SECS {
                        return;
                    }
                    end.set(Some(b));
                    SyncCommand::SetLoop { start: a, end: b }.run();
                    SyncCommand::SeekTo(a).run();
                },
                "B"
            }
            if start().is_some() {
                button {
                    class: "ctrl-btn practice-btn",
                    title: "Stop looping",
                    aria_label: "Clear loop",
                    onclick: move |_| {
                        start.set(None);
                        end.set(None);
                        SyncCommand::ClearLoop.run();
                    },
                    "✕"
                }
            }
            span { class: "practice-label", role: "status", "{label}" }
            TempoTools { song_id: song.id.clone(), fallback_sec, part_start: start() }
            a {
                class: "ctrl-btn practice-chords",
                href: chords_search_url(&song),
                target: "_blank",
                rel: "noopener noreferrer",
                title: "Search the web for this song's chords (opens a new tab)",
                "Chords ↗"
            }
            ChordLane { song_id: song.id.clone(), fallback_sec }
        }
    }
}
