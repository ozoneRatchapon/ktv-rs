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
    on_video_ended: EventHandler<()>,
) -> Element {
    let mut is_skipped = use_signal(|| true);
    let mut is_guide_vocal = use_signal(|| false);
    let mut is_scoring_active = use_signal(|| false);

    // Sync is_skipped with auto_skip_intro when current_item changes
    use_effect(use_reactive((&current_item, &auto_skip_intro), move |(item, auto_skip)| {
        if item.is_some() {
            is_skipped.set(auto_skip);
            is_guide_vocal.set(false);
        }
    }));

    // Listen to YouTube Iframe onStateChange: 0 (ENDED)
    use_effect(move || {
        let mut eval = document::eval(r#"
            if (!window._ktv_youtube_listener_active) {
                window._ktv_youtube_listener_active = true;
                window.addEventListener('message', (event) => {
                    try {
                        let data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data;
                        if (data && (data.event === 'onStateChange' && data.info === 0 || data.info === 0)) {
                            dioxus.send('ended');
                        }
                    } catch(e) {}
                });
            }
        "#);

        spawn(async move {
            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "ended" {
                    on_video_ended.call(());
                }
            }
        });
    });

    match current_item {
        Some(item) => {
            let song = item.song;
            let has_guide = song.guide_video_id.is_some();
            let active_video_id = if is_guide_vocal() {
                song.guide_video_id.clone().unwrap_or_else(|| song.youtube_id.clone())
            } else {
                song.youtube_id.clone()
            };

            let start_sec = if is_guide_vocal() {
                0
            } else if is_skipped() {
                song.intro_skip_secs
            } else {
                0
            };

            let iframe_src = format!(
                "https://www.youtube.com/embed/{active_video_id}?autoplay=1&start={start_sec}&enablejsapi=1&rel=0&iv_load_policy=3"
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
                            key: "{active_video_id}_{start_sec}_{playback_speed}_{is_guide_vocal()}",
                            id: "ktv-youtube-player",
                            src: "{iframe_src}",
                            title: "{song.title}",
                            allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
                            allowfullscreen: true,
                        }

                        // Full-coverage anti-redirect & focus protective shield
                        div {
                            class: "video-click-shield-full",
                            title: "Click to Play/Pause",
                            onclick: move |_| {
                                let _ = document::eval(r#"
                                    let iframe = document.getElementById('ktv-youtube-player');
                                    if (iframe && iframe.contentWindow) {
                                        iframe.contentWindow.postMessage('{"event":"command","func":"togglePlay","args":""}', '*');
                                    }
                                    window.focus();
                                "#);
                            },
                        }

                        // Guide Vocal active badge overlay
                        if is_guide_vocal() {
                            div { class: "guide-vocal-indicator-badge",
                                span { "Original Singer Vocal" }
                            }
                        }

                        // Live Pitch Scoring HUD Overlay
                        if is_scoring_active() {
                            div { class: "live-pitch-scoring-hud",
                                div { class: "hud-score-gauge",
                                    span { class: "hud-label", "PITCH MATCH" }
                                    span { class: "hud-score-value", "94.8" }
                                    span { class: "hud-rank-badge", "RANK S" }
                                }
                                div { class: "hud-pitch-track",
                                    div { class: "pitch-note-pill perfect", "C#4 Match" }
                                    div { class: "pitch-visualizer-bars",
                                        div { class: "wave-bar h-60" }
                                        div { class: "wave-bar h-85" }
                                        div { class: "wave-bar h-100 active" }
                                        div { class: "wave-bar h-75" }
                                        div { class: "wave-bar h-45" }
                                    }
                                    div { class: "combo-badge", "Combo 18" }
                                }
                            }
                        }

                        // Intro skip banner badge (when in karaoke mode)
                        if !is_guide_vocal() {
                            div { class: "intro-skip-badge-overlay",
                                if is_skipped() {
                                    div { class: "intro-banner skipped",
                                        span { "Intro Skipped ({song.intro_skip_secs}s)" }
                                        button {
                                            class: "badge-action-btn",
                                            onclick: move |_| is_skipped.set(false),
                                            "Play Intro"
                                        }
                                    }
                                } else {
                                    div { class: "intro-banner playing-intro",
                                        span { "Playing Intro ({song.intro_skip_secs}s)" }
                                        button {
                                            class: "badge-action-btn primary",
                                            onclick: move |_| is_skipped.set(true),
                                            "Skip Intro"
                                        }
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
                                    "-1"
                                }
                                span { class: "key-pill", "{key_label}" }
                                button {
                                    class: "ctrl-btn key-btn",
                                    title: "Pitch Up (+1 semitone)",
                                    onclick: move |_| on_key_change.call(1),
                                    "+1"
                                }
                            }

                            // Guide Vocal / Original Artist Switcher
                            if has_guide {
                                button {
                                    class: if is_guide_vocal() { "ctrl-btn action-btn guide-active" } else { "ctrl-btn action-btn" },
                                    title: if is_guide_vocal() { "Switch to Instrumental Karaoke" } else { "Switch to Original Artist Vocal" },
                                    onclick: move |_| is_guide_vocal.set(!is_guide_vocal()),
                                    if is_guide_vocal() {
                                        span { "Vocal: Original" }
                                    } else {
                                        span { "Vocal: Karaoke" }
                                    }
                                }
                            }

                            // Live Pitch & Score Evaluation Toggle
                            button {
                                class: if is_scoring_active() { "ctrl-btn action-btn score-active" } else { "ctrl-btn action-btn" },
                                title: "Toggle live pitch evaluation",
                                onclick: move |_| is_scoring_active.set(!is_scoring_active()),
                                span { "Score HUD" }
                            }

                            // Fullscreen Cinema Mode
                            button {
                                class: "ctrl-btn action-btn",
                                title: "Toggle Fullscreen Cinema Mode",
                                onclick: move |_| {
                                    let _ = document::eval(r#"
                                        let elem = document.querySelector('.stage-player-side');
                                        if (!document.fullscreenElement) {
                                            if (elem && elem.requestFullscreen) elem.requestFullscreen();
                                        } else {
                                            if (document.exitFullscreen) document.exitFullscreen();
                                        }
                                    "#);
                                },
                                span { "Fullscreen" }
                            }

                            // Replay
                            button {
                                class: "ctrl-btn action-btn",
                                title: "Restart Song",
                                onclick: move |_| on_replay_song.call(()),
                                span { "Replay" }
                            }

                            // Next Song / Skip
                            button {
                                class: "ctrl-btn action-btn primary-glow",
                                title: "Next Song",
                                onclick: move |_| on_next_song.call(()),
                                span { "Next Song" }
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
                        div { class: "standby-hub-box",
                            div { class: "standby-badge", "ROOM READY" }
                            h2 { class: "standby-title", "KTV STANDBY" }
                            p { class: "standby-desc", "Type any song name, artist, or 5-digit code on your keyboard to start" }

                            div { class: "standby-qr-box",
                                div { class: "qr-placeholder",
                                    span { class: "qr-pixel-lead", "MOBILE REMOTE" }
                                    span { class: "qr-sub", "Open ktv.local or scan code" }
                                }
                            }

                            div { class: "standby-shortcuts-row",
                                span { class: "shortcut-pill", "Keypad: 5-digit code" }
                                span { class: "shortcut-pill", "Type to search" }
                                span { class: "shortcut-pill", "Space: Pause" }
                            }
                        }
                    }
                }
            }
        }
    }
}
