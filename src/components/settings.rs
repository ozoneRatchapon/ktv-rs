use dioxus::prelude::*;

use crate::storage;
use crate::tip;
use crate::types::AppSettings;

#[component]
pub fn Settings(
    settings: Signal<AppSettings>,
) -> Element {
    let current_settings = settings();
    // Two taps: a stray tap must not wipe favourites and scores
    let mut confirm_clear = use_signal(|| false);
    let wallet_status = match tip::parse_wallet(&current_settings.tip.wallet) {
        Ok(_) => "Tip QR shows under the player. Your address and every tip memo are public on-chain.",
        Err(err) => err.message(),
    };
    let rpc_status = match (tip::parse_rpc_url(&current_settings.tip.rpc_url), current_settings.tip.rpc_endpoint()) {
        (Err(err), _) => err.message(),
        (Ok(_), Some(_)) => "Tips are confirmed on-chain and shown under the QR",
        (Ok(_), None) => "Mainnet needs a Helius RPC URL to confirm tips (the public one blocks browsers)",
    };

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
                            id: "default_intro_skip",
                            name: "default_intro_skip",
                            aria_label: "Default skip duration in seconds",
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

                // Guide timing tools
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Guide Timing Tools" }
                        div { class: "setting-sub", "Line up an original-vocal MV with the karaoke video by ear (saved on this device); also shows the Queue tab's End Song test button" }
                    }
                    button {
                        class: if current_settings.show_timing_tools { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| {
                            let mut s = settings();
                            s.show_timing_tools = !s.show_timing_tools;
                            settings.set(s);
                        },
                        if current_settings.show_timing_tools { "ON" } else { "OFF" }
                    }
                }

                // Tip wallet (plan 003): Solana Pay QR under the player
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Tip Wallet (Solana)" }
                        div { class: "setting-sub", "{wallet_status}" }
                    }
                    input {
                        id: "tip_wallet",
                        name: "tip_wallet",
                        aria_label: "Solana wallet address for tips",
                        class: "text-input",
                        r#type: "text",
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "Solana address",
                        value: "{current_settings.tip.wallet}",
                        oninput: move |evt| {
                            let mut s = settings();
                            s.tip.wallet = evt.value().trim().to_string();
                            settings.set(s);
                        }
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Tip Network" }
                        div { class: "setting-sub", "Devnet for demos; switch to Mainnet only to take real USDC" }
                    }
                    button {
                        class: if current_settings.tip.cluster == tip::SolanaCluster::Mainnet { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| {
                            let mut s = settings();
                            s.tip.cluster = s.tip.cluster.toggled();
                            settings.set(s);
                        },
                        "{current_settings.tip.cluster.label()}"
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Tip RPC (optional)" }
                        div { class: "setting-sub", "{rpc_status}" }
                    }
                    input {
                        id: "tip_rpc",
                        name: "tip_rpc",
                        aria_label: "Solana RPC URL for tip checks",
                        class: "text-input",
                        r#type: "url",
                        autocomplete: "off",
                        spellcheck: "false",
                        placeholder: "https://mainnet.helius-rpc.com/?api-key=…",
                        value: "{current_settings.tip.rpc_url}",
                        oninput: move |evt| {
                            let mut s = settings();
                            s.tip.rpc_url = evt.value().trim().to_string();
                            settings.set(s);
                        }
                    }
                }

                // Room Name
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "Room Name" }
                        div { class: "setting-sub", "Display identifier for this KTV room" }
                    }
                    input {
                        id: "room_name",
                        name: "room_name",
                        aria_label: "Room name",
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
                            "This application is a non-commercial educational showcase and technical portfolio demonstrating high-performance web engineering with Dioxus 0.7 and WebAssembly. No audio or video files are hosted on our servers. Videos load in YouTube's privacy-enhanced mode (youtube-nocookie.com); YouTube may still store data in your browser once playback starts, under Google's privacy policy. All media playback is streamed directly via the official YouTube Embed API in compliance with YouTube Terms of Service. All rights, trademarks, and royalties remain with the respective artists and record labels (GMM Grammy, Genie Records, What The Duck, Muzik Move, RS, Smallroom)."
                        }
                        p {
                            strong { "Source code (AGPL-3.0): " }
                            a {
                                href: "https://github.com/ozoneRatchapon/ktv-rs",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "repo-link",
                                "github.com/ozoneRatchapon/ktv-rs"
                            }
                        }
                        p {
                            strong { "Privacy: " }
                            "Settings, queue, favourites, scores and guide timings stay in this browser; mic audio never leaves it. "
                            a { href: "/privacy.html", target: "_blank", class: "repo-link", "Read the privacy note" }
                        }
                        div { class: "clear-data-row",
                            button {
                                class: if confirm_clear() { "ctrl-btn danger-btn" } else { "ctrl-btn" },
                                onclick: move |_| {
                                    if confirm_clear() {
                                        storage::clear_all();
                                        storage::reload_page();
                                    } else {
                                        confirm_clear.set(true);
                                    }
                                },
                                if confirm_clear() { "Tap again to clear everything" } else { "Clear my data on this device" }
                            }
                            if confirm_clear() {
                                button { class: "ctrl-btn", onclick: move |_| confirm_clear.set(false), "Cancel" }
                            }
                        }
                    }
                }
            }
        }
    }
}
