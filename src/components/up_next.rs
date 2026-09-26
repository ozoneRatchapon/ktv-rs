use dioxus::prelude::*;

use crate::types::QueueItem;

/// Songs shown in TV mode's "Up next" strip.
const SHOWN: usize = 3;

/// TV mode: the next songs under the player, readable from across the room.
#[component]
pub fn UpNext(queue: Vec<QueueItem>) -> Element {
    rsx! {
        div { class: "up-next", aria_label: "Up next",
            span { class: "up-next-label", "UP NEXT" }
            if queue.is_empty() {
                span { class: "up-next-empty", "Queue is empty: Auto-DJ picks the next song" }
            }
            for item in queue.iter().take(SHOWN) {
                div { key: "{item.queue_id}", class: "up-next-item",
                    span { class: "up-next-code", "#{item.song.code}" }
                    span { class: "up-next-title", lang: "th", "{item.song.title}" }
                    span { class: "up-next-artist", lang: "th", "{item.song.artist}" }
                }
            }
            if queue.len() > SHOWN {
                span { class: "up-next-more", "+{queue.len() - SHOWN} more" }
            }
        }
    }
}
