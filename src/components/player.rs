use dioxus::prelude::*;
use futures_util::StreamExt;
use crate::sync::{self, SyncCommand, SyncEvent, GUIDE_FRAME_ID, KARAOKE_FRAME_ID};
use crate::components::guide_timing::GuideTiming;
use crate::components::pitch_meter::PitchMeter;
use crate::score::TakeResult;
use crate::timing::SavedTiming;
use crate::types::{GuideTrack, QueueItem};

#[component]
pub fn Player(
    current_item: Option<QueueItem>,
    auto_skip_intro: bool,
    mut is_skipped: Signal<bool>,
    playback_speed: f32,
    /// Changes on every song start or replay (restarts the tuning score).
    take_started_at: f64,
    on_next_song: EventHandler<()>,
    on_replay_song: EventHandler<()>,
    on_key_change: EventHandler<i32>,
    on_video_ended: EventHandler<()>,
    show_timing_tools: bool,
    /// The current song's guide timing comes from this device, not the catalog.
    saved_timing: SavedTiming,
    on_save_guide: EventHandler<GuideTrack>,
    on_revert_guide: EventHandler<()>,
    on_take_end: EventHandler<TakeResult>,
) -> Element {
    let mut is_guide_vocal = use_signal(|| false);
    let mut is_guide_failed = use_signal(|| false);
    let mut is_paused = use_signal(|| false);
    let mut current_playback_sec = use_signal(|| 0u64);
    // What the sync core was last loaded with: (queue id, intro skip, guide video) and the guide mapping
    let mut loaded = use_signal(|| None::<(u64, bool, Option<String>)>);
    let mut loaded_mapping = use_signal(|| (0.0f32, 1.0f32));

    // Sync core must exist before the effects below issue commands, so install during the first render
    use_hook(move || {
        let mut events = sync::install();
        spawn(async move {
            while let Some(msg) = events.next().await {
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

    // Reload the sync core only for a new song (or intro setting, or guide video). A key change or a saved
    // guide timing also rewrites current_item, and must keep the vocal choice and position.
    use_effect(use_reactive((&current_item, &auto_skip_intro), move |(item, auto_skip)| {
        let Some(it) = item else { return };
        let mapping = it.song.guide.as_ref().map_or((0.0, 1.0), |g| (g.offset_secs, g.rate));
        let identity = (it.queue_id, auto_skip, it.song.guide.as_ref().map(|g| g.video_id.clone()));
        if loaded.peek().as_ref() == Some(&identity) {
            if *loaded_mapping.peek() != mapping {
                loaded_mapping.set(mapping);
                let (offset_secs, rate) = mapping;
                SyncCommand::SetMapping { offset_secs, rate }.run();
            }
            return;
        }
        loaded.set(Some(identity));
        loaded_mapping.set(mapping);
        is_skipped.set(auto_skip);
        is_guide_vocal.set(false);
        is_guide_failed.set(false);
        current_playback_sec.set(it.song.start_sec(auto_skip));
        let (offset_secs, rate) = mapping;
        SyncCommand::LoadSong { offset_secs, rate }.run();
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
            let timing_song = song.clone();
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

            // Guide player stays loaded (muted, paused) so the vocal switch is instant
            let guide_iframe_src = song.guide.as_ref().map(|g| {
                let guide_id = &g.video_id;
                format!("https://www.youtube-nocookie.com/embed/{guide_id}?autoplay=0&mute=1&enablejsapi=1&controls=0&rel=0&iv_load_policy=3")
            });

            // YouTube embed policy: nothing may be drawn in front of either player, and a player may only play
            // while visible (>= 200x200). So the guide sits beside the karaoke video, and opens only while it plays.
            rsx! {
                div { class: "player-container",
                    div { class: "video-stage",
                        div { class: "video-frame-wrapper",
                            iframe {
                                key: "{active_video_id}_{start_sec}_{playback_speed}",
                                id: KARAOKE_FRAME_ID,
                                src: "{iframe_src}",
                                title: "{song.title}",
                                allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
                                allowfullscreen: true,
                            }
                        }

                        if let Some(guide_src) = guide_iframe_src {
                            div {
                                class: if is_guide_vocal() { "guide-pane open" } else { "guide-pane" },
                                aria_hidden: if !is_guide_vocal() { "true" },
                                iframe {
                                    key: "guide_{song.id}_{guide_src}",
                                    id: GUIDE_FRAME_ID,
                                    src: "{guide_src}",
                                    title: "Original singer vocal guide",
                                    tabindex: if !is_guide_vocal() { "-1" },
                                    allow: "autoplay; encrypted-media",
                                }
                                span { class: "guide-pane-label", "Original Singer Vocal" }
                            }
                        }
                    }

                    // Bottom Player Bar
                    div { class: "player-bottom-bar",
                        // Intro skip banner (karaoke mode only)
                        if !is_guide_vocal() {
                            div { class: "player-status-row",
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

                        // KTV Interactive Timeline Scrubber Row
                        div { class: "ktv-scrubber-container",
                            span { class: "time-text current-time", "{format_time(current_playback_sec())}" }
                            div { class: "slider-wrapper",
                                input {
                                    id: "playback_scrubber",
                                    name: "playback_scrubber",
                                    aria_label: "Playback position",
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
                                    h2 { class: "now-title", lang: "th", "{song.title}" }
                                    p { class: "now-artist", lang: "th", "{song.artist} • {song.channel}" }
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

                                PitchMeter { take: take_started_at, song: song.clone(), on_take_end }

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
                                    onclick: move |_| crate::browser::toggle_fullscreen(".stage-player-side"),
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

                    if show_timing_tools {
                        GuideTiming {
                            song: timing_song,
                            is_guide_vocal,
                            saved_timing,
                            on_save: on_save_guide,
                            on_revert: on_revert_guide,
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
