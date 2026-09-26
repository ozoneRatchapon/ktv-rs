use dioxus::prelude::*;

use crate::score::TakeResult;

fn score_text(result: &TakeResult) -> String {
    result.score().map_or("—".to_string(), |s| s.to_string())
}

/// Result of the take that just ended: top of the control column, never over a player.
#[component]
pub fn ScoreCard(result: TakeResult, on_close: EventHandler<()>) -> Element {
    let notes = result.notes;
    let cents = result.mean_abs_cents;
    let verdict = match result.score() {
        Some(90..) => "Spot on!",
        Some(70..=89) => "Nicely in tune",
        Some(40..=69) => "Getting there",
        Some(_) => "Keep practising",
        None => "Hold a few more notes for a score",
    };
    rsx! {
        section { class: "score-card", role: "status", aria_label: "Last song result",
            div { class: "score-card-value", "{score_text(&result)}" }
            div { class: "score-card-body",
                div { class: "score-card-verdict", "{verdict}" }
                div { class: "score-card-song", lang: "th", "{result.title} · {result.artist}" }
                div { class: "score-card-detail", "Tuning · {notes} held notes · on average {cents:.0}¢ off" }
            }
            button { class: "ctrl-btn", aria_label: "Close result", onclick: move |_| on_close.call(()), "Close" }
        }
    }
}

/// Recent takes on this device, newest first.
#[component]
pub fn ScoreHistory(results: Vec<TakeResult>) -> Element {
    rsx! {
        div { class: "score-history",
            h4 { class: "dj-heading", "RECENT SCORES" }
            if results.is_empty() {
                p { class: "dj-sub", "Turn on the mic while you sing: each finished song's tuning score shows up here." }
            }
            for (i, result) in results.iter().enumerate() {
                div { key: "{i}", class: "score-row",
                    span { class: "score-row-value", "{score_text(result)}" }
                    span { class: "score-row-song", lang: "th", "{result.title}" }
                    span { class: "score-row-detail", "{result.notes} notes · {result.mean_abs_cents:.0}¢" }
                }
            }
        }
    }
}
