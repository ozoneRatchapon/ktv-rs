use dioxus::prelude::*;

use crate::library::Library;
use crate::search;
use crate::types::Song;

/// Results listed beside the fullscreen player; more means typing on.
const SHOWN: usize = 8;

/// Search results for the fullscreen player, where the songbook panel is out of view.
/// Rendered while the player is fullscreen and there is a query; CSS lays it out beside the video
/// (never over it: YouTube forbids drawing in front of an embedded player).
#[component]
pub fn QuickSearch(
    catalog: Vec<Song>,
    library: Library,
    mut search_query: Signal<String>,
    on_play_song: EventHandler<Song>,
    on_queue_song: EventHandler<Song>,
    on_queue_next_song: EventHandler<Song>,
) -> Element {
    let query = search_query();
    let hits = search::search(catalog.iter().chain(library.songs()), library, &query);
    let total = hits.songs.len();

    rsx! {
        aside { class: "quick-search", aria_label: "Search results",
            div { class: "quick-search-head",
                span { class: "quick-search-query", lang: "th", "🔍 {query}" }
                button {
                    class: "clear-search-btn",
                    title: "Clear search",
                    onclick: move |_| search_query.set(String::new()),
                    "✕"
                }
            }
            if let Some(retyped) = &hits.retyped {
                p { class: "retyped-text", role: "status",
                    "Other keyboard layout: "
                    strong { lang: "th", "{retyped}" }
                }
            }
            p { class: "count-text",
                match total {
                    0 => "No songs found".to_string(),
                    n if n > SHOWN => format!("Top {SHOWN} of {n}: keep typing to narrow it down"),
                    1 => "1 Song".to_string(),
                    n => format!("{n} Songs"),
                }
            }
            for song in hits.songs.iter().copied().take(SHOWN) {
                div { key: "{song.id}", class: "quick-search-item",
                    div { class: "quick-search-song",
                        span { class: "song-code-tag", "#{song.code}" }
                        span { class: "quick-search-title", lang: "th", "{song.title}" }
                        span { class: "quick-search-artist", lang: "th", "{song.artist}" }
                    }
                    div { class: "card-actions",
                        button {
                            class: "card-btn play-now",
                            title: "Play immediately",
                            onclick: {
                                let s = song.clone();
                                move |_| on_play_song.call(s.clone())
                            },
                            "Play"
                        }
                        button {
                            class: "card-btn queue-next",
                            title: "Insert as next song",
                            onclick: {
                                let s = song.clone();
                                move |_| on_queue_next_song.call(s.clone())
                            },
                            "Insert"
                        }
                        button {
                            class: "card-btn queue-add",
                            title: "Add to end of queue",
                            onclick: {
                                let s = song.clone();
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
