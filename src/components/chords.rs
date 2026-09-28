use dioxus::prelude::*;

use crate::browser;
use crate::chords::{Chord, ChordChart, ChordCharts};
use crate::storage::{self, CHORDS_KEY};
use crate::sync;

/// How often the lane checks which chord is sounding (it re-renders only when that changes).
const TICK_MS: i32 = 100;
/// Display transpose range, in semitones.
const MAX_SHIFT: i32 = 11;

fn format_mark(secs: f64) -> String {
    let tenths = (secs.max(0.0) * 10.0).round() as u64;
    format!("{:02}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

/// A stored chord name shifted for display (stored names are always valid; anything else is shown as is).
fn shown(name: &str, shift: i32) -> String {
    name.parse::<Chord>().map_or_else(|_| name.to_string(), |c| c.transposed(shift).to_string())
}

/// Chords by ear: a musician taps chord changes while the song plays; the lane then shows the chord sounding now
/// and the next one. Saved on this device per song (`ktv.chords.v1`); nothing is looked up or analysed.
#[component]
pub fn ChordLane(
    song_id: String,
    /// Current karaoke second, used when the sync core cannot give a finer one.
    fallback_sec: u64,
) -> Element {
    let mut charts = use_signal(|| {
        let stored = storage::load::<ChordCharts>(CHORDS_KEY).unwrap_or_default();
        stored.into_iter().map(|(id, chart)| (id, chart.sanitized())).filter(|(_, c)| !c.is_empty()).collect::<ChordCharts>()
    });
    use_effect(move || storage::save(CHORDS_KEY, &*charts.read()));
    let mut editing = use_signal(|| false);
    let mut shift = use_signal(|| 0i32);
    let mut typed = use_signal(String::new);
    let mut notice = use_signal(|| None::<String>);
    let mut confirm_clear = use_signal(|| false);
    // (sounding, next) mark indexes; written only when they change
    let mut position = use_signal(|| (None::<usize>, None::<usize>));

    // For handlers and the tick; the render reads the prop
    let mut song = use_signal(|| song_id.clone());
    use_effect(use_reactive!(|song_id| {
        song.set(song_id);
        confirm_clear.set(false);
        notice.set(None);
    }));
    let now = move || sync::karaoke_time().unwrap_or(fallback_sec as f64);
    let mut refresh = move || {
        let at = charts.peek().get(&*song.peek()).map(|c| c.position(now()));
        let at = at.unwrap_or((None, None));
        if *position.peek() != at {
            position.set(at);
        }
    };
    use_future(move || async move {
        loop {
            browser::sleep_ms(TICK_MS).await;
            refresh();
        }
    });

    let mut mark = move |name: &str| match name.parse::<Chord>() {
        Ok(chord) => {
            let at = now();
            charts.write().entry(song.peek().clone()).or_default().insert(at, chord);
            notice.set(Some(format!("{chord} at {}", format_mark(at))));
            typed.set(String::new());
            refresh();
        }
        Err(err) => notice.set(Some(err.to_string())),
    };

    let chart: ChordChart = charts.read().get(&song_id).cloned().unwrap_or_default();
    let shift_now = shift();
    let (sounding, next) = position();
    let sounding = sounding.and_then(|i| chart.marks().get(i)).map(|m| shown(&m.chord, shift_now));
    let next = next.and_then(|i| chart.marks().get(i)).map(|m| shown(&m.chord, shift_now));
    let shift_label = match shift_now {
        0 => "Key: as entered".to_string(),
        n => format!("Key: {n:+}"),
    };

    rsx! {
        div { class: "chord-lane",
            if !chart.is_empty() {
                div { class: "chord-now", aria_live: "off",
                    span { class: "chord-now-name", title: "Chord now", "{sounding.as_deref().unwrap_or(\"–\")}" }
                    if let Some(next) = next {
                        span { class: "chord-next", "then {next}" }
                    }
                }
                div { class: "chord-shift",
                    button {
                        class: "ctrl-btn practice-btn",
                        aria_label: "Show chords a semitone lower",
                        disabled: shift_now <= -MAX_SHIFT,
                        onclick: move |_| shift -= 1,
                        "−"
                    }
                    span { class: "chord-shift-label", "{shift_label}" }
                    button {
                        class: "ctrl-btn practice-btn",
                        aria_label: "Show chords a semitone higher",
                        disabled: shift_now >= MAX_SHIFT,
                        onclick: move |_| shift += 1,
                        "+"
                    }
                }
            }
            button {
                class: if editing() { "ctrl-btn practice-btn guide-active" } else { "ctrl-btn practice-btn" },
                aria_pressed: "{editing()}",
                title: "Enter this song's chord changes by ear while it plays (saved on this device)",
                onclick: move |_| editing.toggle(),
                if chart.is_empty() { "Chords by ear" } else { "Edit chords" }
            }
            if editing() {
                div { class: "chord-editor",
                    div { class: "chord-entry",
                        input {
                            id: "chord_name",
                            name: "chord_name",
                            class: "chord-input",
                            aria_label: "Chord name, e.g. Am, F#m7, G/B",
                            placeholder: "Am, F#m7, G/B",
                            autocomplete: "off",
                            spellcheck: false,
                            value: "{typed}",
                            oninput: move |e| typed.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    let name = typed();
                                    mark(&name);
                                }
                            },
                        }
                        button {
                            class: "ctrl-btn practice-btn",
                            title: "The typed chord starts here (Enter does the same)",
                            onclick: move |_| {
                                let name = typed();
                                mark(&name);
                            },
                            "Mark here"
                        }
                    }
                    if !chart.is_empty() {
                        div { class: "chord-palette", role: "group", aria_label: "Chords used so far: tap one when it comes",
                            for name in chart.palette().into_iter().map(str::to_string) {
                                button {
                                    key: "{name}",
                                    class: "ctrl-btn practice-btn",
                                    title: "{name} starts here",
                                    onclick: move |_| mark(&name),
                                    "{name}"
                                }
                            }
                        }
                        ol { class: "chord-marks", aria_label: "Chord changes",
                            for (i, m) in chart.marks().iter().enumerate() {
                                li { key: "{m.at_secs}",
                                    span { class: "chord-mark-time", "{format_mark(m.at_secs)}" }
                                    span { class: "chord-mark-name", "{m.chord}" }
                                    button {
                                        class: "chord-mark-remove",
                                        aria_label: "Remove {m.chord} at {format_mark(m.at_secs)}",
                                        onclick: move |_| {
                                            let id = song.peek().clone();
                                            let mut all = charts.write();
                                            if let Some(c) = all.get_mut(&id) {
                                                c.remove(i);
                                                if c.is_empty() {
                                                    all.remove(&id);
                                                }
                                            }
                                            drop(all);
                                            refresh();
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                        div { class: "chord-actions",
                            button {
                                class: "ctrl-btn practice-btn",
                                title: "Copy this chart as JSON (backup, or to share)",
                                onclick: {
                                    let json = chart.to_json();
                                    move |_| {
                                        let json = json.clone();
                                        spawn(async move {
                                            let copied = browser::copy_text(&json).await;
                                            notice.set(Some(if copied { "Chart copied" } else { "Copy blocked by the browser (needs https)" }.to_string()));
                                        });
                                    }
                                },
                                "Copy chart"
                            }
                            button {
                                class: "ctrl-btn practice-btn",
                                onclick: move |_| {
                                    if !confirm_clear() {
                                        confirm_clear.set(true);
                                        return;
                                    }
                                    charts.write().remove(&*song.peek());
                                    confirm_clear.set(false);
                                    notice.set(Some("Chart cleared".to_string()));
                                    refresh();
                                },
                                if confirm_clear() { "Tap again to clear" } else { "Clear chart" }
                            }
                        }
                    }
                    if let Some(msg) = notice() {
                        p { class: "timing-notice", aria_live: "polite", "{msg}" }
                    }
                }
            }
        }
    }
}
