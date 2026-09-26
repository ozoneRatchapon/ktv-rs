use dioxus::prelude::*;
use crate::search;
use crate::types::Song;

#[component]
pub fn CatalogView(
    catalog: Vec<Song>,
    mut search_query: Signal<String>,
    on_play_song: EventHandler<Song>,
    on_queue_song: EventHandler<Song>,
    on_queue_next_song: EventHandler<Song>,
) -> Element {
    let mut selected_category = use_signal(|| "All".to_string());

    let categories = crate::catalog::get_categories();

    let hits = {
        let cat = selected_category();
        let in_category: Vec<Song> = catalog.iter().filter(|s| cat == "All" || s.category == cat).cloned().collect();
        search::search(&in_category, &search_query())
    };
    let filtered_songs = hits.songs;

    rsx! {
        div { class: "catalog-container",
            // Search and Category Bar
            div { class: "catalog-filter-bar",
                div { class: "search-input-wrapper",
                    input {
                        id: "song_search",
                        name: "song_search",
                        aria_label: "Search songs",
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Search by title, artist, or 5-digit code...",
                        value: "{search_query()}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                    if !search_query().is_empty() {
                        button {
                            class: "clear-search-btn",
                            onclick: move |_| search_query.set(String::new()),
                            "✕"
                        }
                    }
                }

                // Category chips
                div { class: "category-chips",
                    for cat in categories {
                        button {
                            key: "{cat}",
                            class: if selected_category() == cat { "chip active" } else { "chip" },
                            onclick: move |_| selected_category.set(cat.to_string()),
                            "{cat}"
                        }
                    }
                }
            }

            // Song list count
            div { class: "catalog-meta-row",
                span { class: "count-text", "{filtered_songs.len()} Songs" }
                if let Some(retyped) = &hits.retyped {
                    span { class: "retyped-text", role: "status",
                        "Keyboard was on the other layout: showing results for "
                        strong { lang: "th", "{retyped}" }
                    }
                }
            }

            // Song list grid
            div { class: "songs-grid",
                for song in filtered_songs {
                    div { key: "{song.id}", class: "song-card",
                        div { class: "card-left",
                            div { class: "song-code-tag", "#{song.code}" }
                            div { class: "song-details",
                                h3 { class: "song-title", lang: "th", "{song.title}" }
                                p { class: "song-artist", lang: "th", "{song.artist} • {song.channel}" }
                                div { class: "song-badges",
                                    span { class: "genre-badge", "{song.category}" }
                                    if song.intro_skip_secs > 0 {
                                        span { class: "intro-badge", "Intro: {song.intro_skip_secs}s" }
                                    }
                                }
                            }
                        }

                        div { class: "card-actions",
                            button {
                                class: "card-btn play-now",
                                title: "Play immediately",
                                onclick: {
                                    let s = song.clone();
                                    move |_| on_play_song.call(s.clone())
                                },
                                span { "Play" }
                            }
                            button {
                                class: "card-btn queue-next",
                                title: "Insert as next song",
                                onclick: {
                                    let s = song.clone();
                                    move |_| on_queue_next_song.call(s.clone())
                                },
                                span { "Insert" }
                            }
                            button {
                                class: "card-btn queue-add",
                                title: "Add to end of queue",
                                onclick: {
                                    let s = song.clone();
                                    move |_| on_queue_song.call(s.clone())
                                },
                                span { "Queue" }
                            }
                        }
                    }
                }
            }
        }
    }
}
