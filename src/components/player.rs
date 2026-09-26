use dioxus::prelude::*;
use crate::sync::{self, SyncCommand, SyncEvent, GUIDE_FRAME_ID, KARAOKE_FRAME_ID};
use crate::components::pitch_meter::PitchMeter;
use crate::types::QueueItem;

#[component]
pub fn Player(
    current_item: Option<QueueItem>,
    auto_skip_intro: bool,
    mut is_skipped: Signal<bool>,
    playback_speed: f32,
    on_next_song: EventHandler<()>,
    on_replay_song: EventHandler<()>,
    on_key_change: EventHandler<i32>,
    on_video_ended: EventHandler<()>,
) -> Element {
    let mut is_guide_vocal = use_signal(|| false);
    let mut is_guide_failed = use_signal(|| false);
    let mut is_paused = use_signal(|| false);
    let mut current_playback_sec = use_signal(|| 0u64);

    // Sync core must exist before the effects below issue commands, so install during the first render
    use_hook(move || {
        let mut eval = sync::install();
        spawn(async move {
            while let Ok(msg) = eval.recv::<String>().await {
                match SyncEvent::parse(&msg) {
                    Some(SyncEvent::Ended) => on_video_ended.call(()),
                    Some(SyncEvent::Time(sec)) => current_playback_sec.set(sec),
                    Some(SyncEvent::Paused(paused)) => is_paused.set(paused),
                    Some(SyncEvent::GuideError(_)) => {
                        is_guide_vocal.set(false);
                        is_guide_failed.set(true);
                    }
                    None => {}
                }
            }
        });
    });

    // Sync is_skipped with auto_skip_intro when current_item changes
    use_effect(use_reactive((&current_item, &auto_skip_intro), move |(item, auto_skip)| {
        if let Some(it) = item {
            is_skipped.set(auto_skip);
            is_guide_vocal.set(false);
            is_guide_failed.set(false);
            current_playback_sec.set(it.song.start_sec(auto_skip));
            let (offset_secs, rate) = it.song.guide.as_ref().map_or((0.0, 1.0), |g| (g.offset_secs, g.rate));
            SyncCommand::LoadSong { offset_secs, rate }.run();
        }
    }));

    // Karaoke iframe remounts whenever its video or start second changes; keep the sync clock in step
    let start_sec = current_item
        .as_ref()
        .map(|it| it.song.start_sec(is_skipped()));
    let video_id = current_item.as_ref().map(|it| it.song.youtube_id.clone());
    use_effect(use_reactive((&video_id, &start_sec), move |(id, sec)| {
        if let (Some(_), Some(sec)) = (id, sec) {
            SyncCommand::SetStart(sec).run();
        }
    }));

    match current_item {
        Some(item) => {
            let song = item.song;
            let has_guide = song.guide.is_some();
            let active_video_id = song.youtube_id.clone();

            let start_sec = start_sec.unwrap_or_default();

            let iframe_src = format!(
                "https://www.youtube-nocookie.com/embed/{active_video_id}?autoplay=1&start={start_sec}&enablejsapi=1&rel=0&iv_load_policy=3"
            );

            let key_label = match item.key_shift {
                k if k > 0 => format!("KEY: +{k} ♯"),
                k if k < 0 => format!("KEY: {k} ♭"),
                _ => "ORIGINAL KEY (±0)".to_string(),
            };

            // Hidden guide player: stays loaded (muted, paused) so the vocal switch is instant
            let guide_iframe_src = song.guide.as_ref().map(|g| {
                let guide_id = &g.video_id;
                format!("https://www.youtube-nocookie.com/embed/{guide_id}?autoplay=0&mute=1&enablejsapi=1&controls=0&rel=0&iv_load_policy=3")
            });

            rsx! {
                div { class: "player-container",
                    div { class: "video-frame-wrapper",
                        if let Some(guide_src) = guide_iframe_src {
                            iframe {
                                key: "guide_{song.id}",
                                id: GUIDE_FRAME_ID,
                                class: "ktv-guide-audio-frame",
                                src: "{guide_src}",
                                title: "Original singer vocal guide",
                                aria_hidden: "true",
                                tabindex: "-1",
                                allow: "autoplay; encrypted-media",
                            }
                        }

                        iframe {
                            key: "{active_video_id}_{start_sec}_{playback_speed}",
                            id: KARAOKE_FRAME_ID,
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
                                SyncCommand::TogglePlayback.run();
                                let _ = document::eval("window.focus();");
                            },
                        }

                        // Guide Vocal active badge overlay with preserved timestamp cue
                        if is_guide_vocal() {
                            div { class: "guide-vocal-indicator-badge",
                                span { "Original Singer Vocal" }
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
                        // KTV Interactive Timeline Scrubber Row
                        div { class: "ktv-scrubber-container",
                            span { class: "time-text current-time", "{format_time(current_playback_sec())}" }
                            div { class: "slider-wrapper",
                                input {
                                    class: "ktv-scrubber-slider",
                                    r#type: "range",
                                    min: "0",
                                    max: "{u64::from(song.duration_secs)}",
                                    value: "{current_playback_sec()}",
                                    oninput: move |evt| {
                                        if let Ok(target) = evt.value().parse::<u64>() {
                                            current_playback_sec.set(target);
                                            SyncCommand::SeekTo(target).run();
                                        }
                                    },
                                }
                            }
                            span { class: "time-text total-time", "{format_time(u64::from(song.duration_secs))}" }
                        }

                        div { class: "player-main-controls-row",
                            div { class: "song-info",
                                div { class: "song-code-pill", "#{song.code}" }
                                div { class: "song-titles",
                                    h2 { class: "now-title", "{song.title}" }
                                    p { class: "now-artist", "{song.artist} • {song.channel}" }
                                }
                            }

                            div { class: "player-quick-controls",
                                // Rewind -10s
                                button {
                                    class: "ctrl-btn jump-btn",
                                    title: "Rewind 10 seconds",
                                    onclick: move |_| {
                                        let curr = current_playback_sec();
                                        let target = curr.saturating_sub(10);
                                        current_playback_sec.set(target);
                                        SyncCommand::SeekTo(target).run();
                                    },
                                    "-10s"
                                }

                                // Forward +10s
                                button {
                                    class: "ctrl-btn jump-btn",
                                    title: "Forward 10 seconds",
                                    onclick: move |_| {
                                        let curr = current_playback_sec();
                                        let target = (curr + 10).min(u64::from(song.duration_secs));
                                        current_playback_sec.set(target);
                                        SyncCommand::SeekTo(target).run();
                                    },
                                    "+10s"
                                }

                                // Key Transpose Buttons
                                div { class: "key-control-group",
                                    button {
                                        class: "ctrl-btn key-btn",
                                        title: "Key -1 (label only: YouTube audio cannot be pitch-shifted)",
                                        onclick: move |_| on_key_change.call(-1),
                                        "-1"
                                    }
                                    span { class: "key-pill", "{key_label}" }
                                    button {
                                        class: "ctrl-btn key-btn",
                                        title: "Key +1 (label only: YouTube audio cannot be pitch-shifted)",
                                        onclick: move |_| on_key_change.call(1),
                                        "+1"
                                    }
                                }

                                // Guide Vocal / Original Artist Switcher with Intro Offset Compensation
                                if has_guide {
                                    button {
                                        class: if is_guide_vocal() { "ctrl-btn action-btn guide-active" } else { "ctrl-btn action-btn" },
                                        disabled: is_guide_failed(),
                                        title: match (is_guide_failed(), is_guide_vocal()) {
                                            (true, _) => "Original vocal video is unavailable on YouTube",
                                            (false, true) => "Switch back to Karaoke",
                                            (false, false) => "Switch to Original Artist Vocal (in-sync)",
                                        },
                                        onclick: move |_| {
                                            // Karaoke video keeps playing (lyrics stay visible); only the audio source swaps
                                            let next = !is_guide_vocal();
                                            is_guide_vocal.set(next);
                                            SyncCommand::SwitchVocal { original: next }.run();
                                        },
                                        if is_guide_failed() {
                                            span { "Vocal: Unavailable" }
                                        } else if is_guide_vocal() {
                                            span { "Vocal: Original" }
                                        } else {
                                            span { "Vocal: Karaoke" }
                                        }
                                    }
                                }

                                PitchMeter {}

                                // Play / Pause Toggle Button
                                button {
                                    class: if is_paused() { "ctrl-btn action-btn pause-active" } else { "ctrl-btn action-btn" },
                                    title: if is_paused() { "Resume Playback (Space)" } else { "Pause Playback (Space)" },
                                    onclick: move |_| {
                                        SyncCommand::TogglePlayback.run();
                                    },
                                    if is_paused() {
                                        span { "Play" }
                                    } else {
                                        span { "Pause" }
                                    }
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
                                    onclick: move |_| {
                                        current_playback_sec.set(start_sec);
                                        is_paused.set(false);
                                        on_replay_song.call(());
                                    },
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
        }
        None => {
            rsx! {
                div { class: "player-container standby-mode",
                    div { class: "standby-content",
                        div { class: "standby-hub-box",
                            div { class: "standby-badge", "ROOM READY" }
                            h2 { class: "standby-title", "KTV STANDBY" }
                            p { class: "standby-desc", "Type any song name, artist, or 5-digit code on your keyboard to start" }

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

fn format_time(total_secs: u64) -> String {
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{m:02}:{s:02}")
}
