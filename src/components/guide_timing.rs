use dioxus::prelude::*;

use crate::sync::{self, SyncCommand};
use crate::timing::{self, SyncMark};
use crate::types::{GuideTrack, Song};
use crate::youtube::parse_video_id;

/// Nudge steps in seconds: coarse to find the line, fine to lock it.
const NUDGES: [f64; 4] = [-1.0, -0.1, 0.1, 1.0];

fn format_mark(secs: f64) -> String {
    let whole = secs.max(0.0) as u64;
    format!("{:02}:{:02}", whole / 60, whole % 60)
}

/// Curator panel: line up the original-vocal MV with the karaoke video by ear, with the embedded players only.
/// Nudges retime the running guide live; Save stores the timing on this device; the JSON goes in `assets/catalog.json`.
#[component]
pub fn GuideTiming(
    song: Song,
    mut is_guide_vocal: Signal<bool>,
    is_overridden: bool,
    on_save: EventHandler<GuideTrack>,
    on_revert: EventHandler<()>,
) -> Element {
    let mut draft = use_signal(|| song.guide.clone());
    let mut marks = use_signal(Vec::<SyncMark>::new);
    let mut monitor = use_signal(|| false);
    let mut video_input = use_signal(String::new);
    let mut notice = use_signal(|| None::<String>);

    // New song: start over (the sync core also drops monitoring on load)
    let song_id = song.id.clone();
    use_effect(use_reactive!(|song_id| {
        let _ = &song_id;
        marks.set(Vec::new());
        monitor.set(false);
        notice.set(None);
        video_input.set(String::new());
    }));
    // Saved or reverted timing becomes the new starting point
    let saved = song.guide.clone();
    use_effect(use_reactive!(|saved| draft.set(saved)));

    let mut retime = move |guide: GuideTrack| {
        SyncCommand::SetMapping { offset_secs: guide.offset_secs, rate: guide.rate }.run();
        draft.set(Some(guide));
    };

    let use_video = move |_| match parse_video_id(&video_input()) {
        Some(id) => {
            // A different MV has its own timeline: start from zero offset and normal speed
            let guide = timing::new_guide(&id);
            marks.set(Vec::new());
            notice.set(Some("Guide video set. Turn on the guide, then nudge until the singer lines up.".to_string()));
            on_save.call(guide);
        }
        None => notice.set(Some("Not a YouTube video link or 11-character video ID".to_string())),
    };

    let mark = move |_| {
        let Some(guide) = draft() else { return };
        spawn(async move {
            match sync::karaoke_time().await {
                Some(secs) => {
                    marks.write().push(timing::mark_at(&guide, secs));
                    notice.set(None);
                }
                None => notice.set(Some("Player is not ready yet".to_string())),
            }
        });
    };

    let fit_speed = move |_| {
        let (Some(guide), Some(first), Some(last)) = (draft(), marks().first().copied(), marks().last().copied()) else {
            return;
        };
        match timing::fit(&guide.video_id, first, last) {
            Ok(fitted) => {
                notice.set(Some(format!("Speed fitted from marks at {} and {}", format_mark(first.karaoke_secs), format_mark(last.karaoke_secs))));
                retime(fitted);
            }
            Err(err) => notice.set(Some(err.to_string())),
        }
    };

    let copy_json = move |_| {
        let Some(guide) = draft() else { return };
        let text = serde_json::to_string(&timing::catalog_snippet(&guide)).unwrap_or_default();
        let _ = document::eval(&format!("navigator.clipboard && navigator.clipboard.writeText({text});"));
        notice.set(Some("Copied: paste it into this song's entry in assets/catalog.json".to_string()));
    };

    let unsaved = draft() != song.guide;
    let min_span = timing::MIN_FIT_SPAN_SECS;

    rsx! {
        div { class: "guide-timing-panel",
            div { class: "timing-head",
                h4 { "Guide timing" }
                span { class: "timing-status",
                    match (unsaved, is_overridden, song.guide.is_some()) {
                        (true, _, _) => "unsaved changes",
                        (false, true, _) => "saved on this device",
                        (false, false, true) => "from catalog",
                        (false, false, false) => "no guide yet",
                    }
                }
            }

            div { class: "timing-row",
                input {
                    class: "text-input timing-video-input",
                    r#type: "text",
                    placeholder: "Original MV link or video ID",
                    value: "{video_input}",
                    oninput: move |evt| video_input.set(evt.value()),
                }
                button { class: "ctrl-btn", onclick: use_video,
                    if song.guide.is_some() { "Change MV" } else { "Use MV" }
                }
            }

            if let Some(guide) = draft() {
                div { class: "timing-row",
                    if !is_guide_vocal() {
                        button {
                            class: "ctrl-btn action-btn",
                            title: "Play the original vocal MV in sync with the karaoke video",
                            onclick: move |_| {
                                is_guide_vocal.set(true);
                                SyncCommand::SwitchVocal { original: true }.run();
                            },
                            "Play guide"
                        }
                    }
                    button {
                        class: if monitor() { "ctrl-btn action-btn guide-active" } else { "ctrl-btn action-btn" },
                        disabled: !is_guide_vocal(),
                        title: "Play the karaoke music under the guide: when they are out of sync you hear an echo",
                        onclick: move |_| {
                            let next = !monitor();
                            monitor.set(next);
                            SyncCommand::SetMonitor(next).run();
                        },
                        if monitor() { "Hear both: On" } else { "Hear both: Off" }
                    }
                }

                div { class: "timing-row timing-nudge",
                    for step in NUDGES {
                        button {
                            class: "ctrl-btn jump-btn",
                            title: if step > 0.0 { "Singer comes in late: jump the guide ahead" } else { "Singer comes in early: hold the guide back" },
                            onclick: {
                                let guide = guide.clone();
                                move |_| retime(timing::nudge(&guide, step))
                            },
                            "{step:+}s"
                        }
                    }
                    span { class: "timing-readout", "offset {guide.offset_secs:+.2}s · speed {guide.rate:.4}" }
                }
                p { class: "timing-hint", "Singer late → press +. Singer early → press −." }

                div { class: "timing-row",
                    button {
                        class: "ctrl-btn action-btn",
                        title: "Record this moment as in sync (use one early and one late in the song)",
                        onclick: mark,
                        "Mark in sync"
                    }
                    for m in marks() {
                        span { class: "timing-mark", "{format_mark(m.karaoke_secs)}" }
                    }
                    if marks().len() >= 2 {
                        button {
                            class: "ctrl-btn action-btn",
                            title: "Fit the speed so the first and last marks both stay in sync",
                            onclick: fit_speed,
                            "Fit speed"
                        }
                    }
                    if !marks().is_empty() {
                        button { class: "ctrl-btn", onclick: move |_| marks.set(Vec::new()), "Clear" }
                    }
                }
                if marks().len() < 2 {
                    p { class: "timing-hint",
                        "Only if it drifts: sync and mark early, then sync and mark again at least {min_span:.0}s later, then Fit speed."
                    }
                }

                div { class: "timing-row",
                    button {
                        class: "ctrl-btn action-btn primary-glow",
                        disabled: !unsaved,
                        onclick: {
                            let guide = guide.clone();
                            move |_| {
                                notice.set(Some("Saved on this device".to_string()));
                                on_save.call(guide.clone());
                            }
                        },
                        "Save"
                    }
                    if is_overridden {
                        button {
                            class: "ctrl-btn",
                            title: "Drop the timing saved on this device and use the catalog's",
                            onclick: move |_| {
                                notice.set(Some("Timing saved on this device removed".to_string()));
                                on_revert.call(());
                            },
                            "Revert"
                        }
                    }
                    button { class: "ctrl-btn", onclick: copy_json, "Copy JSON" }
                }
                pre { class: "timing-snippet", "{timing::catalog_snippet(&guide)}" }
            }

            if let Some(msg) = notice() {
                p { class: "timing-notice", aria_live: "polite", "{msg}" }
            }
        }
    }
}
