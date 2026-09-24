use dioxus::prelude::*;
use crate::types::Song;

#[component]
pub fn Remote(
    catalog: Vec<Song>,
    on_play_by_code: EventHandler<String>,
    on_queue_by_code: EventHandler<String>,
    on_skip_song: EventHandler<()>,
    on_replay_song: EventHandler<()>,
    on_key_shift: EventHandler<i32>,
    on_reset_key: EventHandler<()>,
    on_speed_change: EventHandler<f32>,
    current_key: i32,
    current_speed: f32,
) -> Element {
    let mut input_code = use_signal(String::new);
    let mut message_feedback = use_signal(|| None::<String>);

    let matched_song = {
        let code = input_code();
        if code.is_empty() {
            None
        } else {
            catalog.iter().find(|s| s.code == code).cloned()
        }
    };


    let backspace = move |_| {
        let mut curr = input_code();
        curr.pop();
        input_code.set(curr);
    };

    let clear = move |_| {
        input_code.set(String::new());
    };

    let play_now = move |_| {
        let code = input_code();
        if !code.is_empty() {
            on_play_by_code.call(code.clone());
            message_feedback.set(Some(format!("กำลังเปิดเพลงรหัส {code}!")));
            input_code.set(String::new());
        }
    };

    let queue_now = move |_| {
        let code = input_code();
        if !code.is_empty() {
            on_queue_by_code.call(code.clone());
            message_feedback.set(Some(format!("จองเพลงรหัส {code} เข้าคิวแล้ว!")));
            input_code.set(String::new());
        }
    };

    rsx! {
        div { class: "remote-container",
            div { class: "remote-panel",
                // Header of Remote
                div { class: "remote-header",
                    div { class: "remote-title-badge", "KTV SMART CONTROLLER" }
                    if let Some(msg) = message_feedback() {
                        div { class: "remote-feedback-toast", "{msg}" }
                    }
                }

                // Digital Code Screen
                div { class: "code-screen",
                    div { class: "screen-label", "ENTER 5-DIGIT SONG CODE" }
                    div { class: "code-display",
                        span { class: "code-digits",
                            if input_code().is_empty() {
                                "_____"
                            } else {
                                "{input_code()}"
                            }
                        }
                    }

                    // Song preview if found
                    if let Some(s) = matched_song {
                        div { class: "preview-card",
                            span { class: "preview-icon", "🎵" }
                            div { class: "preview-text",
                                div { class: "preview-title", "{s.title}" }
                                div { class: "preview-artist", "{s.artist}" }
                            }
                        }
                    }
                }

                // 10-Key Numpad
                div { class: "numpad-grid",
                    for d in ['1', '2', '3', '4', '5', '6', '7', '8', '9'] {
                        button {
                            key: "{d}",
                            class: "numpad-key",
                            onclick: move |_| {
                                let mut curr = input_code();
                                if curr.len() < 5 {
                                    curr.push(d);
                                    input_code.set(curr);
                                }
                            },
                            "{d}"
                        }
                    }
                    button {
                        class: "numpad-key fn-key",
                        onclick: backspace,
                        "⌫"
                    }
                    button {
                        class: "numpad-key",
                        onclick: move |_| {
                            let mut curr = input_code();
                            if curr.len() < 5 {
                                curr.push('0');
                                input_code.set(curr);
                            }
                        },
                        "0"
                    }
                    button {
                        class: "numpad-key fn-key",
                        onclick: clear,
                        "CLEAR"
                    }
                }

                // Code Actions: Play Now vs Add to Queue
                div { class: "code-actions-row",
                    button {
                        class: "code-action-btn primary-glow",
                        onclick: play_now,
                        span { "▶️ ร้องเลย (Play)" }
                    }
                    button {
                        class: "code-action-btn secondary-btn",
                        onclick: queue_now,
                        span { "➕ จองเพลง (Queue)" }
                    }
                }

                // Pitch Transpose Control Section
                div { class: "remote-section",
                    div { class: "section-title", "🎚️ KEY TRANSPOSE (ปรับคีย์)" }
                    div { class: "key-transpose-row",
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(-2),
                            "♭♭ -2"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(-1),
                            "♭ -1"
                        }
                        button {
                            class: if current_key == 0 { "pitch-btn active" } else { "pitch-btn" },
                            onclick: move |_| on_reset_key.call(()),
                            "ORIGINAL (±0)"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(1),
                            "♯ +1"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(2),
                            "♯♯ +2"
                        }
                    }
                }

                // Tempo & Song Controls
                div { class: "remote-section",
                    div { class: "section-title", "⚡ PLAYBACK CONTROLS (ควบคุมเพลง)" }
                    div { class: "transport-row",
                        button {
                            class: "transport-btn warning",
                            onclick: move |_| on_replay_song.call(()),
                            "🔄 ร้องใหม่"
                        }
                        button {
                            class: "transport-btn danger",
                            onclick: move |_| on_skip_song.call(()),
                            "⏭️ ข้ามเพลง"
                        }
                    }

                    div { class: "speed-row",
                        span { class: "speed-label", "TEMPO:" }
                        button {
                            class: if (current_speed - 0.9).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(0.9),
                            "0.9x ช้า"
                        }
                        button {
                            class: if (current_speed - 1.0).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(1.0),
                            "1.0x ปกติ"
                        }
                        button {
                            class: if (current_speed - 1.1).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(1.1),
                            "1.1x เร็ว"
                        }
                    }
                }
            }
        }
    }
}
