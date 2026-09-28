use dioxus::prelude::*;

use crate::score::{LeaderRow, MedleyTotal, TakeResult, MAX_NAME_CHARS};

/// Recent takes listed under the leaderboard (the device keeps more for the party).
const SHOWN_RECENT: usize = 20;

fn score_text(result: &TakeResult) -> String {
    result.score().map_or("—".to_string(), |s| s.to_string())
}

/// Result of the take that just ended: top of the control column, never over a player. The singer can put a
/// name on it for the party leaderboard (kept on this device only).
#[component]
pub fn ScoreCard(
    result: TakeResult,
    /// After a medley's last part: the whole medley's score, shown as the card's big number.
    medley_total: Option<MedleyTotal>,
    /// Names used before, newest first: one tap instead of typing.
    singers: Vec<String>,
    on_name: EventHandler<Option<String>>,
    on_close: EventHandler<()>,
) -> Element {
    let notes = result.notes;
    let cents = result.mean_abs_cents;
    let name_id = match result.part {
        Some(part) if part > 1 => format!("singer_name_{part}"),
        _ => "singer_name".to_string(),
    };
    let shown = medley_total.as_ref().map_or(result.score(), |t| Some(t.score));
    let verdict = match shown {
        Some(90..) => "Spot on!",
        Some(70..=89) => "Nicely in tune",
        Some(40..=69) => "Getting there",
        Some(_) => "Keep practising",
        None => "Hold a few more notes for a score",
    };
    let mut typed = use_signal(String::new);
    let mut save = move || {
        let name = typed();
        on_name.call(Some(name));
        typed.set(String::new());
    };
    rsx! {
        section { class: "score-card", role: "status", aria_label: "Last song result",
            div { class: "score-card-value", {shown.map_or("—".to_string(), |s| s.to_string())} }
            div { class: "score-card-body",
                div { class: "score-card-verdict", "{verdict}" }
                if let Some(total) = &medley_total {
                    div { class: "score-card-song score-card-medley", lang: "th", "Medley · {total.title}" }
                    div { class: "score-card-detail",
                        "Mean of {total.scored} of {total.count} parts · last part {score_text(&result)}: {result.title}"
                    }
                } else {
                    div { class: "score-card-song", lang: "th", "{result.title} · {result.artist}" }
                    if let Some(m) = &result.medley {
                        div { class: "score-card-detail", "Medley {m.index + 1}/{m.count} · {m.title}" }
                    }
                }
                if let Some(part) = result.part {
                    div { class: "score-card-detail score-card-part", if part == 1 { "Duet · singer 1 (left mic)" } else { "Duet · singer 2 (right mic)" } }
                }
                div { class: "score-card-detail", "Tuning · {notes} held notes · on average {cents:.0}¢ off" }
                if let Some(melody) = result.melody_score {
                    div { class: "score-card-detail score-card-melody", "Melody {melody} · right notes, any octave" }
                }
                if result.score().is_some() {
                    match result.singer.clone() {
                        Some(name) => rsx! {
                            div { class: "score-card-singer",
                                span { "Sung by " strong { "{name}" } }
                                button {
                                    class: "chord-mark-remove",
                                    aria_label: "Remove the singer's name",
                                    onclick: move |_| on_name.call(None),
                                    "✕"
                                }
                            }
                        },
                        None => rsx! {
                            div { class: "score-card-singer",
                                input {
                                    // One field per card: a duet shows two
                                    id: "{name_id}",
                                    name: "{name_id}",
                                    class: "chord-input",
                                    aria_label: "Who sang? (for the party leaderboard, kept on this device)",
                                    placeholder: "Who sang?",
                                    maxlength: "{MAX_NAME_CHARS}",
                                    autocomplete: "off",
                                    value: "{typed}",
                                    oninput: move |e| typed.set(e.value()),
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            save();
                                        }
                                    },
                                }
                                button {
                                    class: "ctrl-btn practice-btn",
                                    disabled: typed().trim().is_empty(),
                                    onclick: move |_| save(),
                                    "Save"
                                }
                                for name in singers {
                                    button {
                                        key: "{name}",
                                        class: "ctrl-btn practice-btn",
                                        title: "Sung by {name}",
                                        onclick: move |_| on_name.call(Some(name.clone())),
                                        "{name}"
                                    }
                                }
                            }
                        },
                    }
                }
            }
            button { class: "ctrl-btn", aria_label: "Close result", onclick: move |_| on_close.call(()), "Close" }
        }
    }
}

/// Tonight's best take per named singer, then recent takes on this device, newest first.
#[component]
pub fn ScoreHistory(results: Vec<TakeResult>, leaders: Vec<LeaderRow>) -> Element {
    rsx! {
        div { class: "score-history",
            if !leaders.is_empty() {
                h4 { class: "dj-heading", "PARTY LEADERBOARD" }
                p { class: "dj-sub", "Best take per singer, last 12 hours. Names stay on this device." }
                ol { class: "leaderboard",
                    for (rank, row) in leaders.iter().enumerate() {
                        li { key: "{row.singer}", class: "score-row",
                            span { class: "leader-rank", "{rank + 1}" }
                            span { class: "score-row-value", "{score_text(&row.best)}" }
                            span { class: "score-row-song",
                                strong { "{row.singer}" }
                                span { lang: "th", " · {row.best.title}" }
                            }
                            span { class: "score-row-detail",
                                if row.takes == 1 { "1 song" } else { "{row.takes} songs" }
                            }
                        }
                    }
                }
            }
            h4 { class: "dj-heading", "RECENT SCORES" }
            if results.is_empty() {
                p { class: "dj-sub", "Turn on the mic while you sing: each finished song's tuning score shows up here." }
            }
            for (i, result) in results.iter().take(SHOWN_RECENT).enumerate() {
                div { key: "{i}", class: "score-row",
                    span { class: "score-row-value", "{score_text(result)}" }
                    span { class: "score-row-song", lang: "th",
                        "{result.title}"
                        if let Some(name) = &result.singer {
                            span { class: "score-row-singer", " · {name}" }
                        }
                    }
                    span { class: "score-row-detail", "{result.notes} notes · {result.mean_abs_cents:.0}¢" }
                }
            }
        }
    }
}
