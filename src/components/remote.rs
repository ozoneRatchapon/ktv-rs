use dioxus::prelude::*;
use crate::catalog::find_song;
use crate::library::Library;
use crate::types::Song;

#[component]
pub fn Remote(
    catalog: Vec<Song>,
    library: Library,
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
            find_song(&catalog, library, |s| s.code == code).cloned()
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
            message_feedback.set(Some(format!("Playing song {code}")));
            input_code.set(String::new());
        }
    };

    let queue_now = move |_| {
        let code = input_code();
        if !code.is_empty() {
            on_queue_by_code.call(code.clone());
            message_feedback.set(Some(format!("Song {code} added to the queue")));
            input_code.set(String::new());
        }
    };

    rsx! {
        div { class: "remote-container",
            div { class: "remote-panel",
                // Header of Remote
                div { class: "remote-header",
                    div { class: "remote-title-badge", "KEYPAD CONTROLLER" }
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
                            div { class: "preview-text",
                                div { class: "preview-title", lang: "th", "{s.title}" }
                                div { class: "preview-artist", lang: "th", "{s.artist} • {s.channel}" }
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
                        class: "code-action-btn primary-btn",
                        onclick: play_now,
                        span { "Play" }
                    }
                    button {
                        class: "code-action-btn secondary-btn",
                        onclick: queue_now,
                        span { "Queue" }
                    }
                }

                // Pitch Transpose Control Section
                div { class: "remote-section",
                    div { class: "section-title", "KEY TRANSPOSE" }
                    div {
                        class: "key-transpose-row",
                        title: "Saved per song as a key label; YouTube audio cannot be pitch-shifted in the browser",
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(-2),
                            "-2"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(-1),
                            "-1"
                        }
                        button {
                            class: if current_key == 0 { "pitch-btn active" } else { "pitch-btn" },
                            onclick: move |_| on_reset_key.call(()),
                            "ORIGINAL"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(1),
                            "+1"
                        }
                        button {
                            class: "pitch-btn",
                            onclick: move |_| on_key_shift.call(2),
                            "+2"
                        }
                    }
                }

                // Tempo & Song Controls
                div { class: "remote-section",
                    div { class: "section-title", "PLAYBACK CONTROLS" }
                    div { class: "transport-row",
                        button {
                            class: "transport-btn warning",
                            onclick: move |_| on_replay_song.call(()),
                            "Replay"
                        }
                        button {
                            class: "transport-btn danger",
                            onclick: move |_| on_skip_song.call(()),
                            "Next Song"
                        }
                    }

                    div { class: "speed-row",
                        span { class: "speed-label", "TEMPO:" }
                        button {
                            class: if (current_speed - 0.9).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(0.9),
                            "0.9x"
                        }
                        button {
                            class: if (current_speed - 1.0).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(1.0),
                            "1.0x"
                        }
                        button {
                            class: if (current_speed - 1.1).abs() < 0.01 { "speed-pill active" } else { "speed-pill" },
                            onclick: move |_| on_speed_change.call(1.1),
                            "1.1x"
                        }
                    }
                }
            }
        }
    }
}
