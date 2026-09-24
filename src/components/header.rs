use dioxus::prelude::*;
use crate::types::KtvTab;

#[component]
pub fn Header(
    active_tab: Signal<KtvTab>,
    queue_len: usize,
    room_name: String,
) -> Element {
    rsx! {
        header { class: "ktv-header",
            div { class: "header-left",
                div { class: "neon-logo",
                    span { class: "neon-icon", "🎤" }
                    div { class: "logo-text-group",
                        span { class: "logo-brand", "NEON KTV" }
                        span { class: "logo-sub", "GMM OFFICIAL FEED" }
                    }
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
                    span { class: "nav-icon", "📖" }
                    span { "Songbook (เลือกเพลง)" }
                }

                button {
                    class: if active_tab() == KtvTab::Queue { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Queue),
                    span { class: "nav-icon", "📋" }
                    span { "Queue (คิวเพลง)" }
                    if queue_len > 0 {
                        span { class: "queue-badge", "{queue_len}" }
                    }
                }

                button {
                    class: if active_tab() == KtvTab::Remote { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Remote),
                    span { class: "nav-icon", "🎛️" }
                    span { "Remote (รีโมท)" }
                }

                button {
                    class: if active_tab() == KtvTab::CustomAdd { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::CustomAdd),
                    span { class: "nav-icon", "➕" }
                    span { "Add URL (เพิ่มลิงก์)" }
                }

                button {
                    class: if active_tab() == KtvTab::Settings { "nav-btn active" } else { "nav-btn" },
                    onclick: move |_| active_tab.set(KtvTab::Settings),
                    span { class: "nav-icon", "⚙️" }
                    span { "Settings" }
                }
            }
        }
    }
}
