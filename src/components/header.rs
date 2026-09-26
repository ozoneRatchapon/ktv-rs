use dioxus::prelude::*;
use crate::types::KtvTab;

#[component]
pub fn Header(
    active_tab: Signal<KtvTab>,
    queue_len: usize,
    room_name: String,
    on_help: EventHandler<()>,
    tv_mode: bool,
    on_toggle_tv: EventHandler<()>,
) -> Element {
    rsx! {
        header { class: "ktv-header",
            div { class: "header-left",
                div { class: "brand-group",
                    span { class: "brand-title", "KTV-RS" }
                    span { class: "brand-sub", "OPEN KARAOKE PLATFORM" }
                }
                div { class: "room-badge",
                    span { class: "status-dot" }
                    span { class: "room-title", "{room_name}" }
                }
            }

            nav { class: "header-nav",
                button {
                    class: if active_tab() == KtvTab::Catalog { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Catalog),
                    span { "Songbook" }
                }

                button {
                    class: if active_tab() == KtvTab::Queue { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Queue),
                    span { "Queue" }
                    if queue_len > 0 {
                        span { class: "queue-badge", "{queue_len}" }
                    }
                }

                button {
                    class: if active_tab() == KtvTab::Remote { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Remote),
                    span { "Keypad" }
                }

                button {
                    class: if active_tab() == KtvTab::CustomAdd { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::CustomAdd),
                    span { "Add URL" }
                }

                button {
                    class: if active_tab() == KtvTab::Settings { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Settings),
                    span { "Settings" }
                }

                button {
                    class: if tv_mode { "nav-btn active" } else { "nav-btn" },
                    title: "TV mode: bigger text and player for a booth or TV screen",
                    aria_pressed: "{tv_mode}",
                    onclick: move |_| on_toggle_tv.call(()),
                    span { "TV" }
                }

                button {
                    class: "nav-btn",
                    title: "Keyboard shortcuts (?)",
                    aria_label: "Keyboard shortcuts",
                    onclick: move |_| on_help.call(()),
                    span { "?" }
                }
            }
        }
    }
}
