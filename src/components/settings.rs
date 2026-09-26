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

            // Developer Portfolio & Legal Transparency Notice
            div { class: "settings-card legal-transparency-card",
                div { class: "settings-header",
                    div {
                        h3 { "Developer Showcase & Transparency" }
                        p { "Architecture overview, privacy disclosure, and copyright attribution" }
                    }
                }

                div { class: "legal-info-content",
                    div { class: "legal-metric-row",
                        div { class: "metric-item",
                            span { class: "metric-num", "100%" }
                            span { class: "metric-desc", "Client-Side Rust WASM" }
                        }
                        div { class: "metric-item",
                            span { class: "metric-num", "0" }
                            span { class: "metric-desc", "First-Party Cookies / Trackers" }
                        }
                        div { class: "metric-item",
                            span { class: "metric-num", "nocookie" }
                            span { class: "metric-desc", "YouTube Privacy-Enhanced Embeds" }
                        }
                    }

                    div { class: "legal-text-block",
                        p {
                            strong { "Legal & Copyright Attribution: " }
                            "This application is a non-commercial educational showcase and technical portfolio demonstrating high-performance web engineering with Dioxus 0.7 and WebAssembly. No audio or video files are hosted on our servers. Videos load in YouTube's privacy-enhanced mode (youtube-nocookie.com); YouTube may still store data in your browser once playback starts, under Google's privacy policy. All media playback is streamed directly via the official YouTube Embed API in compliance with YouTube Terms of Service. All rights, trademarks, and royalties remain with the respective artists and record labels (GMM Grammy, Genie Records, What The Duck)."
                        }
                        p {
                            strong { "Open Source Repository: " }
                            a {
                                href: "https://github.com/ozoneRatchapon/ktv-rs",
                                target: "_blank",
                                class: "repo-link",
                                "github.com/ozoneRatchapon/ktv-rs"
                            }
                        }
                    }
                }
            }
        }
    }
}
