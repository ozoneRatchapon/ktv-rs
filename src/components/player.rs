use dioxus::prelude::*;
use crate::types::QueueItem;

#[component]
pub fn Player(
    current_item: Option<QueueItem>,
    auto_skip_intro: bool,
    playback_speed: f32,
    on_next_song: EventHandler<()>,
    on_replay_song: EventHandler<()>,
    on_key_change: EventHandler<i32>,
) -> Element {
    let mut is_skipped = use_signal(|| true);

    // Sync is_skipped with auto_skip_intro when current_item changes
    use_effect(use_reactive((&current_item, &auto_skip_intro), move |(item, auto_skip)| {
        if item.is_some() {
            is_skipped.set(auto_skip);
        }
    }));

    match current_item {
        Some(item) => {
            let song = item.song;
            let start_sec = if is_skipped() { song.intro_skip_secs } else { 0 };
            let video_id = song.youtube_id.clone();
            let iframe_src = format!(
                "https://www.youtube.com/embed/{video_id}?autoplay=1&start={start_sec}&enablejsapi=1&rel=0&iv_load_policy=3"
            );

            let key_label = match item.key_shift {
                k if k > 0 => format!("KEY: +{k} ♯"),
                k if k < 0 => format!("KEY: {k} ♭"),
                _ => "ORIGINAL KEY (±0)".to_string(),
            };

            rsx! {
                div { class: "player-container",
                    div { class: "video-frame-wrapper",
                        iframe {
                            key: "{video_id}_{start_sec}_{playback_speed}",
                            id: "ktv-youtube-player",
                            src: "{iframe_src}",
                            title: "{song.title}",
                            allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
                            allowfullscreen: true,
                        }

                        // Intro skip banner badge
                        div { class: "intro-skip-badge-overlay",
                            if is_skipped() {
                                div { class: "intro-banner skipped",
                                    span { class: "badge-icon", "⚡" }
                                    span { "Skipped Intro ({song.intro_skip_secs}s)" }
                                    button {
                                        class: "badge-action-btn",
                                        onclick: move |_| is_skipped.set(false),
                                        "Play Intro"
                                    }
                                }
                            } else {
                                div { class: "intro-banner playing-intro",
                                    span { class: "badge-icon", "⏱️" }
                                    span { "Playing Intro bumper ({song.intro_skip_secs}s)" }
                                    button {
                                        class: "badge-action-btn primary",
                                        onclick: move |_| is_skipped.set(true),
                                        "Skip Intro ⏩"
                                    }
                                }
                            }
                        }
                    }

                    // Bottom Player Bar
                    div { class: "player-bottom-bar",
                        div { class: "song-info",
                            div { class: "song-code-pill", "#{song.code}" }
                            div { class: "song-titles",
                                h2 { class: "now-title", "{song.title}" }
                                p { class: "now-artist", "{song.artist} • {song.channel}" }
                            }
                        }

                        div { class: "player-quick-controls",
                            // Key Transpose Buttons
                            div { class: "key-control-group",
                                button {
                                    class: "ctrl-btn key-btn",
                                    title: "Pitch Down (-1 semitone)",
                                    onclick: move |_| on_key_change.call(-1),
                                    "♭ -1"
                                }
                                span { class: "key-pill", "{key_label}" }
                                button {
                                    class: "ctrl-btn key-btn",
                                    title: "Pitch Up (+1 semitone)",
                                    onclick: move |_| on_key_change.call(1),
                                    "♯ +1"
                                }
                            }

                            // Replay
                            button {
                                class: "ctrl-btn action-btn",
                                title: "Restart Song (ร้องใหม่)",
                                onclick: move |_| on_replay_song.call(()),
                                span { "🔄 ร้องใหม่" }
                            }

                            // Next Song / Skip
                            button {
                                class: "ctrl-btn action-btn primary-glow",
                                title: "Skip Song (ตัดเพลงถัดไป)",
                                onclick: move |_| on_next_song.call(()),
                                span { "ข้ามเพลง ⏭️" }
                            }
                        }
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "player-container standby-mode",
                    div { class: "standby-content",
                        div { class: "standby-pulse", "🎤" }
                        h2 { class: "standby-title", "KARAOKE ROOM STANDBY" }
                        p { class: "standby-desc", "ไม่มีเพลงกำลังเล่น เลือกเพลงจาก Songbook หรือกดรหัสเพลงในรีโมทเพื่อเริ่มร้อง" }
                    }
                }
            }
        }
    }
}
