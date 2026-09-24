use dioxus::prelude::*;
use crate::types::QueueItem;

#[component]
pub fn QueueView(
    queue: Vec<QueueItem>,
    current_item: Option<QueueItem>,
    on_skip: EventHandler<()>,
    on_remove: EventHandler<u64>,
    on_move_up: EventHandler<usize>,
    on_move_down: EventHandler<usize>,
    on_clear_queue: EventHandler<()>,
    on_adjust_item_key: EventHandler<(u64, i32)>,
) -> Element {
    rsx! {
        div { class: "queue-container",
            // Currently Playing Card
            div { class: "now-playing-section",
                div { class: "section-badge", "NOW SINGING (กำลังร้องอยู่)" }
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
                                h3 { class: "now-song-title", "{curr.song.title}" }
                                p { class: "now-song-artist", "{curr.song.artist}" }
                            }
                        }
                        div { class: "now-actions",
                            button {
                                class: "skip-btn primary-glow",
                                onclick: move |_| on_skip.call(()),
                                "ข้ามเพลงนี้ ⏭️"
                            }
                        }
                    }
                } else {
                    div { class: "empty-now-playing",
                        p { "ไม่มีเพลงกำลังร้องอยู่ เลือกเพลงเพื่อเริ่มร้องทันที" }
                    }
                }
            }

            // Up Next Queue Header
            div { class: "queue-header-row",
                div { class: "queue-title-group",
                    h3 { class: "queue-heading", "UP NEXT IN QUEUE (คิวเพลงรอร้อง)" }
                    span { class: "queue-count-pill", "{queue.len()} เพลง" }
                }

                if !queue.is_empty() {
                    button {
                        class: "clear-all-btn",
                        onclick: move |_| on_clear_queue.call(()),
                        "ลบคิวทั้งหมด 🗑️"
                    }
                }
            }

            // Queue List
            if queue.is_empty() {
                div { class: "empty-queue-box",
                    div { class: "empty-icon", "🎵" }
                    h4 { "ยังไม่มีเพลงในคิว" }
                    p { "กด 'จองเพลง' หรือ 'แทรกคิว' จากหน้า Songbook ได้เลย" }
                }
            } else {
                div { class: "queue-list",
                    for (idx, item) in queue.iter().enumerate() {
                        div { key: "{item.queue_id}", class: "queue-item-card",
                            div { class: "item-rank", "#{idx + 1}" }

                            div { class: "item-info",
                                div { class: "item-code", "#{item.song.code}" }
                                div { class: "item-title", "{item.song.title}" }
                                div { class: "item-artist", "{item.song.artist}" }
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
        }
    }
}
