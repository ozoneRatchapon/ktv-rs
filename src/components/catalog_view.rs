use dioxus::prelude::*;
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

    let filtered_songs = {
        let q = search_query().trim().to_lowercase();
        let cat = selected_category();

        catalog
            .iter()
            .filter(|s| {
                let matches_cat = cat == "All" || s.category == cat;
                let matches_query = q.is_empty()
                    || s.title.to_lowercase().contains(&q)
                    || s.artist.to_lowercase().contains(&q)
                    || s.code.contains(&q);
                matches_cat && matches_query
            })
            .cloned()
            .collect::<Vec<Song>>()
    };

    rsx! {
        div { class: "catalog-container",
            // Search and Category Bar
            div { class: "catalog-filter-bar",
                div { class: "search-input-wrapper",
                    input {
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
            }

            // Song list grid
            div { class: "songs-grid",
                for song in filtered_songs {
                    div { key: "{song.id}", class: "song-card",
                        div { class: "card-left",
                            div { class: "song-code-tag", "#{song.code}" }
                            div { class: "song-details",
                                h3 { class: "song-title", "{song.title}" }
                                p { class: "song-artist", "{song.artist} • {song.channel}" }
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
