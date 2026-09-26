use dioxus::prelude::*;

/// Keyboard shortcuts handled by the global key listener in `main.rs` (keep the two in step).
pub const SHORTCUTS: [(&str, &str); 6] = [
    ("A–Z, 0–9", "Type to search songs, artists or 5-digit codes"),
    ("Backspace", "Delete the last search letter"),
    ("Esc", "Clear the search"),
    ("Space", "Pause / play"),
    ("← / →", "Back / forward 5 seconds"),
    ("?", "Show or hide these shortcuts"),
];

/// Shortcut list shown in the control column, never over the players (YouTube forbids overlays).
#[component]
pub fn ShortcutHelp(on_close: EventHandler<()>) -> Element {
    rsx! {
        section { class: "shortcut-help", aria_label: "Keyboard shortcuts",
            div { class: "shortcut-help-head",
                h3 { "Keyboard shortcuts" }
                button {
                    class: "ctrl-btn",
                    aria_label: "Close keyboard shortcuts",
                    onclick: move |_| on_close.call(()),
                    "Close"
                }
            }
            dl { class: "shortcut-list",
                for (keys, action) in SHORTCUTS {
                    div { class: "shortcut-item",
                        dt { kbd { "{keys}" } }
                        dd { "{action}" }
                    }
                }
            }
        }
    }
}
