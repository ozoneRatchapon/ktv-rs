use dioxus::prelude::*;

use crate::browser;

/// How long a tip toast stays up: long enough to read across a room, short enough not to cover the next one.
const SHOW_MS: i32 = 8_000;

/// Plan 003 S3: a garland toast when a tip lands (Thai stages hang a flower garland on a singer they love).
/// Key it by the tip's signature so each new tip restarts the timer.
#[component]
pub fn TipToast(text: String, on_close: EventHandler<()>) -> Element {
    use_future(move || async move {
        browser::sleep_ms(SHOW_MS).await;
        on_close.call(());
    });
    rsx! {
        div { class: "tip-toast", role: "status",
            span { class: "tip-garland", aria_hidden: "true", "🌼🌸🌼" }
            span { "{text}" }
            button { class: "toast-close-btn", aria_label: "Close", onclick: move |_| on_close.call(()), "✕" }
        }
    }
}
