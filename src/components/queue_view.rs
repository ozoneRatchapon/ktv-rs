use dioxus::prelude::*;
use crate::recommendation::AnticipatedRecommendation;
use crate::types::{QueueItem, Song};

#[component]
pub fn QueueView(
    queue: Vec<QueueItem>,
    current_item: Option<QueueItem>,
    anticipated: Vec<AnticipatedRecommendation>,
    commitment_hash: String,
    on_skip: EventHandler<()>,
    on_remove: EventHandler<u64>,
    on_move_up: EventHandler<usize>,
    on_move_down: EventHandler<usize>,
    on_clear_queue: EventHandler<()>,
    on_adjust_item_key: EventHandler<(u64, i32)>,
    on_queue_song: EventHandler<Song>,
    on_play_song: EventHandler<Song>,
    on_simulate_end: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "queue-container",
            // Currently Playing Card
            div { class: "now-playing-section",
                div { class: "now-playing-header-row",
                    div { class: "section-badge", "NOW SINGING" }
                    if current_item.is_some() {
                        button {
                            class: "test-end-btn",
                            title: "Simulate video ending to test auto-advance to next song",
                            onclick: move |_| on_simulate_end.call(()),
                            "End Song"
                        }
                    }
                }

                if let Some(curr) = current_item {
                    div { class: "now-playing-card",
                        div { class: "now-left",
                            div { class: "live-indicator",
                                span { class: "wave-bar" }
                                span { class: "wave-bar" }
                                span { class: "wave-bar" }
                            }
                            div { class: "now-meta",
                                div { class: "now-code", "#{curr.song.code}" }
                                h3 { class: "now-song-title", lang: "th", "{curr.song.title}" }
                                p { class: "now-song-artist", lang: "th", "{curr.song.artist} • {curr.song.channel}" }
                            }
                        }
                        div { class: "now-actions",
                            button {
                                class: "skip-btn primary-glow",
                                onclick: move |_| on_skip.call(()),
                                "Skip"
                            }
                        }
                    }
                } else {
                    div { class: "empty-now-playing",
                        p { "No song currently playing. Select a song from the catalog to begin." }
                    }
                }
            }

            // Up Next Queue Header
            div { class: "queue-header-row",
                div { class: "queue-title-group",
                    h3 { class: "queue-heading", "QUEUE" }
                    span { class: "queue-count-pill", "{queue.len()} songs" }
                }

                if !queue.is_empty() {
                    button {
                        class: "clear-all-btn",
                        onclick: move |_| on_clear_queue.call(()),
                        "Clear Queue"
                    }
                }
            }

            // Queue List
            if queue.is_empty() {
                div { class: "empty-queue-box",
                    h4 { "Queue is empty" }
                    p { "Auto-DJ will automatically transition to anticipated songs." }
                }
            } else {
                div { class: "queue-list",
                    for (idx, item) in queue.iter().enumerate() {
                        div { key: "{item.queue_id}", class: "queue-item-card",
                            div { class: "item-rank", "#{idx + 1}" }

                            div { class: "item-info",
                                div { class: "item-code", "#{item.song.code}" }
                                div { class: "item-title", lang: "th", "{item.song.title}" }
                                div { class: "item-artist", lang: "th", "{item.song.artist} • {item.song.channel}" }
                            }

                            // Pre-set key for this song
                            div { class: "item-key-adjust",
                                button {
                                    class: "item-key-btn",
                                    title: "Lower key",
                                    onclick: {
                                        let qid = item.queue_id;
                                        move |_| on_adjust_item_key.call((qid, -1))
                                    },
                                    "♭"
                                }
                                span { class: "item-key-val",
                                    if item.key_shift > 0 {
                                        "+{item.key_shift}"
                                    } else {
                                        "{item.key_shift}"
                                    }
                                }
                                button {
                                    class: "item-key-btn",
                                    title: "Raise key",
                                    onclick: {
                                        let qid = item.queue_id;
                                        move |_| on_adjust_item_key.call((qid, 1))
                                    },
                                    "♯"
                                }
                            }

                            // Reordering and delete actions
                            div { class: "item-actions",
                                if idx > 0 {
                                    button {
                                        class: "item-order-btn",
                                        title: "Move up",
                                        onclick: move |_| on_move_up.call(idx),
                                        "▲"
                                    }
                                }
                                if idx + 1 < queue.len() {
                                    button {
                                        class: "item-order-btn",
                                        title: "Move down",
                                        onclick: move |_| on_move_down.call(idx),
                                        "▼"
                                    }
                                }
                                button {
                                    class: "item-delete-btn",
                                    title: "Remove from queue",
                                    onclick: {
                                        let qid = item.queue_id;
                                        move |_| on_remove.call(qid)
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }
            }

            // Smart Auto-DJ Anticipation Section (katgpt-sleep substrate)
            div { class: "auto-dj-section",
                div { class: "auto-dj-header",
                    div { class: "auto-dj-title-group",
                        div {
                            h4 { class: "dj-heading", "AUTO-DJ SUGGESTIONS" }
                            p { class: "dj-sub", "Predictive queue based on listening history & dwell time" }
                        }
                    }
                    if !commitment_hash.is_empty() {
                        span {
                            class: "blake3-badge",
                            title: "BLAKE3 cryptographic state commitment",
                            "BLAKE3: {&commitment_hash[..8]}"
                        }
                    }
                }

                div { class: "anticipated-list",
                    for rec in anticipated.iter() {
                        div { key: "{rec.song.id}", class: "anticipated-card",
                            div { class: "antic-left",
                                div { class: "antic-score-ring",
                                    span { class: "score-val", "{(rec.predictability * 100.0) as u32}%" }
                                    span { class: "score-lbl", "match" }
                                }
                                div { class: "antic-meta",
                                    div { class: "antic-title", lang: "th", "{rec.song.title}" }
                                    div { class: "antic-artist", lang: "th", "{rec.song.artist} • {rec.song.category}" }
                                    div { class: "antic-reason", "{rec.reason}" }
                                }
                            }
                            div { class: "antic-actions",
                                button {
                                    class: "antic-btn play",
                                    onclick: {
                                        let s = rec.song.clone();
                                        move |_| on_play_song.call(s.clone())
                                    },
                                    "Play"
                                }
                                button {
                                    class: "antic-btn queue",
                                    onclick: {
                                        let s = rec.song.clone();
                                        move |_| on_queue_song.call(s.clone())
                                    },
                                    "Queue"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
