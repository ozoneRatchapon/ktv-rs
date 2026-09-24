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
    let mut is_paused = use_signal(|| false);
    let mut preserved_switch_sec = use_signal(|| 0u64);
    let mut current_playback_sec = use_signal(|| 0u64);

    // Sync is_skipped with auto_skip_intro when current_item changes
    use_effect(use_reactive((&current_item, &auto_skip_intro), move |(item, auto_skip)| {
        if let Some(it) = item {
            is_skipped.set(auto_skip);
            is_guide_vocal.set(false);
            preserved_switch_sec.set(0);
            current_playback_sec.set(if auto_skip { u64::from(it.song.intro_skip_secs) } else { 0 });
        }
    }));

    // Listen to YouTube Iframe onStateChange: 0 (ENDED) and infoDelivery currentTime
    use_effect(move || {
        let mut eval = document::eval(r#"
            if (!window._ktv_youtube_listener_active) {
                window._ktv_youtube_listener_active = true;
                window.addEventListener('message', (event) => {
                    try {
                        let data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data;
                        if (data && data.info && typeof data.info.currentTime === 'number') {
                            window._ktv_video_current_time = data.info.currentTime;
                        }
                        if (data && (data.event === 'onStateChange' && data.info === 0 || data.info === 0)) {
                            dioxus.send('ended');
                        }
                    } catch(e) {}
                });
            }

            window.addEventListener('ktv-pause-state', (e) => {
                dioxus.send('PAUSE_STATE:' + (e.detail ? '1' : '0'));
            });

            window._ktv_toggle_playback = function(forcedState) {
                let iframe = document.getElementById('ktv-youtube-player');
                if (!iframe || !iframe.contentWindow) return;
                let nextState = (typeof forcedState === 'boolean') ? forcedState : !window._ktv_is_paused;
                window._ktv_is_paused = nextState;
                if (!nextState) {
                    window._ktv_video_mount_time = Date.now();
                    window._ktv_current_start_sec = window._ktv_video_current_time || 0;
                }
                let cmd = nextState ? 'pauseVideo' : 'playVideo';
                iframe.contentWindow.postMessage(JSON.stringify({
                    event: 'command',
                    func: cmd,
                    args: []
                }), '*');
                window.dispatchEvent(new CustomEvent('ktv-pause-state', { detail: nextState }));
            };

            if (window._ktv_progress_ticker) {
                clearInterval(window._ktv_progress_ticker);
            }
            window._ktv_progress_ticker = setInterval(() => {
                if (window._ktv_is_paused) return;
                let iframe = document.getElementById('ktv-youtube-player');
                if (iframe && iframe.contentWindow) {
                    try {
                        iframe.contentWindow.postMessage('{"event":"listening"}', '*');
                    } catch(e) {}
                }
                let cur = window._ktv_video_current_time;
                if (!cur || cur <= 0) {
                    let elapsed = (Date.now() - (window._ktv_video_mount_time || Date.now())) / 1000;
                    cur = Math.max(0, elapsed + (window._ktv_current_start_sec || 0));
                }
                dioxus.send('TIME:' + Math.floor(cur));
            }, 1000);
        "#);

        spawn(async move {
            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "ended" {
                    on_video_ended.call(());
                } else if let Some(time_str) = msg.strip_prefix("TIME:") {
                    if let Ok(sec) = time_str.parse::<u64>() {
                        current_playback_sec.set(sec);
                    }
                } else if let Some(pause_str) = msg.strip_prefix("PAUSE_STATE:") {
                    is_paused.set(pause_str == "1");
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

            let start_sec = if preserved_switch_sec() > 0 {
                preserved_switch_sec()
            } else if is_guide_vocal() {
                0
            } else if is_skipped() {
                u64::from(song.intro_skip_secs)
            } else {
                0
            };

            // Keep JS mount timer in sync with current start_sec
            use_effect(use_reactive((&active_video_id, &start_sec), move |(_, sec)| {
                let js = format!(r#"
                    window._ktv_video_mount_time = Date.now();
                    window._ktv_current_start_sec = {sec};
                    window._ktv_video_current_time = {sec};
                "#);
                let _ = document::eval(&js);
            }));

            let iframe_src = format!(
                "https://www.youtube.com/embed/{active_video_id}?autoplay=1&start={start_sec}&enablejsapi=1&rel=0&iv_load_policy=3"
            );

            let key_label = match item.key_shift {
                k if k > 0 => format!("KEY: +{k} ♯"),
                k if k < 0 => format!("KEY: {k} ♭"),
                _ => "ORIGINAL KEY (±0)".to_string(),
            };

            let cue_text = if preserved_switch_sec() > 0 {
                let s = preserved_switch_sec();
                format!("Original Singer Vocal • Synced {:02}:{:02}", s / 60, s % 60)
            } else {
                "Original Singer Vocal".to_string()
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
                                    if (typeof window._ktv_toggle_playback === 'function') {
                                        window._ktv_toggle_playback();
                                    }
                                    window.focus();
                                "#);
                            },
                        }

                        // Guide Vocal active badge overlay with preserved timestamp cue
                        if is_guide_vocal() {
                            div { class: "guide-vocal-indicator-badge",
                                span { "{cue_text}" }
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
                                            onclick: move |_| {
                                                preserved_switch_sec.set(0);
                                                is_skipped.set(false);
                                            },
                                            "Play Intro"
                                        }
                                    }
                                } else {
                                    div { class: "intro-banner playing-intro",
                                        span { "Playing Intro ({song.intro_skip_secs}s)" }
                                        button {
                                            class: "badge-action-btn primary",
                                            onclick: move |_| {
                                                preserved_switch_sec.set(0);
                                                is_skipped.set(true);
                                            },
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
                                            preserved_switch_sec.set(target);
                                            seek_video_to(target);
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
                                        preserved_switch_sec.set(target);
                                        seek_video_to(target);
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
                                        preserved_switch_sec.set(target);
                                        seek_video_to(target);
                                    },
                                    "+10s"
                                }

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

                                // Guide Vocal / Original Artist Switcher with Intro Offset Compensation
                                if has_guide {
                                    button {
                                        class: if is_guide_vocal() { "ctrl-btn action-btn guide-active" } else { "ctrl-btn action-btn" },
                                        title: if is_guide_vocal() { "Switch back to Karaoke" } else { "Switch to Original Artist Vocal (in-sync)" },
                                        onclick: move |_| {
                                            let mut eval_time = document::eval(r#"
                                                let cur = window._ktv_video_current_time;
                                                if (!cur || cur <= 0) {
                                                    let elapsed = (Date.now() - (window._ktv_video_mount_time || Date.now())) / 1000;
                                                    cur = Math.max(0, elapsed + (window._ktv_current_start_sec || 0));
                                                }
                                                dioxus.send(Math.floor(cur).toString());
                                            "#);
                                            let offset = song.guide_offset_secs as i64;
                                            let dur = u64::from(song.duration_secs);
                                            spawn(async move {
                                                if let Ok(sec_str) = eval_time.recv::<String>().await {
                                                    let raw_sec = sec_str.parse::<u64>().unwrap_or_else(|_| current_playback_sec());
                                                    if !is_guide_vocal() {
                                                        // Karaoke -> Official MV: add individual song offset
                                                        let mv_sec = (raw_sec as i64 + offset).clamp(0, dur as i64) as u64;
                                                        preserved_switch_sec.set(mv_sec);
                                                        current_playback_sec.set(mv_sec);
                                                        is_guide_vocal.set(true);
                                                    } else {
                                                        // Official MV -> Karaoke: subtract individual song offset
                                                        let karaoke_sec = (raw_sec as i64 - offset).clamp(0, dur as i64) as u64;
                                                        preserved_switch_sec.set(karaoke_sec);
                                                        current_playback_sec.set(karaoke_sec);
                                                        is_guide_vocal.set(false);
                                                    }
                                                }
                                            });
                                        },
                                        if is_guide_vocal() {
                                            span { "Vocal: Original" }
                                        } else {
                                            span { "Vocal: Karaoke" }
                                        }
                                    }
                                }

                                // Play / Pause Toggle Button
                                button {
                                    class: if is_paused() { "ctrl-btn action-btn pause-active" } else { "ctrl-btn action-btn" },
                                    title: if is_paused() { "Resume Playback (Space)" } else { "Pause Playback (Space)" },
                                    onclick: move |_| {
                                        let _ = document::eval(r#"
                                            if (typeof window._ktv_toggle_playback === 'function') {
                                                window._ktv_toggle_playback();
                                            }
                                        "#);
                                    },
                                    if is_paused() {
                                        span { "Play" }
                                    } else {
                                        span { "Pause" }
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
                                    onclick: move |_| {
                                        preserved_switch_sec.set(0);
                                        current_playback_sec.set(0);
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

fn format_time(total_secs: u64) -> String {
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{m:02}:{s:02}")
}

fn seek_video_to(target: u64) {
    let js = format!(
        "window._ktv_video_mount_time = Date.now(); \
         window._ktv_current_start_sec = {target}; \
         window._ktv_video_current_time = {target}; \
         let iframe = document.getElementById('ktv-youtube-player'); \
         if (iframe && iframe.contentWindow) {{ \
             iframe.contentWindow.postMessage(JSON.stringify({{ \
                 event: 'command', \
                 func: 'seekTo', \
                 args: [{target}, true] \
             }}), '*'); \
         }}"
    );
    let _ = document::eval(&js);
}

