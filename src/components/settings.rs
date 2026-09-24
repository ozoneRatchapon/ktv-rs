use dioxus::prelude::*;
use crate::types::AppSettings;

#[component]
pub fn Settings(
    settings: Signal<AppSettings>,
) -> Element {
    let current_settings = settings();

    rsx! {
        div { class: "settings-container",
            div { class: "settings-card",
                div { class: "settings-header",
                    div {
                        h3 { "System Settings" }
                        p { "Player configuration and intro skip preferences" }
                    }
                }

                // Auto-skip intro toggle
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Auto Skip Platform Intro" }
                        div { class: "setting-sub", "Automatically skip channel bumper intro on playback start" }
                    }
                    button {
                        class: if current_settings.auto_skip_intro { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| {
                            let mut s = settings();
                            s.auto_skip_intro = !s.auto_skip_intro;
                            settings.set(s);
                        },
                        if current_settings.auto_skip_intro { "ON" } else { "OFF" }
                    }
                }

                // Default Intro Skip Offset
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Default Skip Duration" }
                        div { class: "setting-sub", "Standard channel intro offset duration in seconds (13s for GMM)" }
                    }
                    div { class: "setting-input-group",
                        input {
                            class: "number-input",
                            r#type: "number",
                            min: "0",
                            max: "40",
                            value: "{current_settings.default_intro_skip_secs}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u32>() {
                                    let mut s = settings();
                                    s.default_intro_skip_secs = val;
                                    settings.set(s);
                                }
                            }
                        }
                        span { class: "input-unit", "sec" }
                    }
                }

                // Room Name
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Room Name" }
                        div { class: "setting-sub", "Display identifier for this KTV room" }
                    }
                    input {
                        class: "text-input",
                        r#type: "text",
                        value: "{current_settings.room_name}",
                        oninput: move |evt| {
                            let mut s = settings();
                            s.room_name = evt.value();
                            settings.set(s);
                        }
                    }
                }
            }
        }
    }
}
