use dioxus::prelude::*;
use crate::catalog::CATEGORIES;
use crate::picks::{Picks, Shelf};
use crate::search;
use crate::types::Song;

#[component]
pub fn CatalogView(
    catalog: Vec<Song>,
    mut search_query: Signal<String>,
    on_play_song: EventHandler<Song>,
    on_queue_song: EventHandler<Song>,
    on_queue_next_song: EventHandler<Song>,
    picks: Picks,
    on_toggle_favourite: EventHandler<String>,
) -> Element {
    let mut shelf = use_signal(|| Shelf::All);
    let shelves: Vec<Shelf> = [Shelf::All, Shelf::Favourites, Shelf::Recent]
        .into_iter()
        .chain(CATEGORIES.into_iter().map(Shelf::Category))
        .collect();

    let hits = search::search(&picks.shelf(&catalog, &shelf()), &search_query());
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
                    for choice in shelves {
                        button {
                            key: "{choice.label()}",
                            class: if shelf() == choice { "chip active" } else { "chip" },
                            aria_pressed: "{shelf() == choice}",
                            onclick: {
                                let choice = choice.clone();
                                move |_| shelf.set(choice.clone())
                            },
                            "{choice.label()}"
                        }
                    }
                }
            }

            // Song list count
            div { class: "catalog-meta-row",
                span { class: "count-text", "{filtered_songs.len()} Songs" }
                if filtered_songs.is_empty() && search_query().is_empty() {
                    match shelf() {
                        Shelf::Favourites => rsx! { span { class: "shelf-hint", "Tap ☆ on a song to keep it here." } },
                        Shelf::Recent => rsx! { span { class: "shelf-hint", "Songs you sing for 30 seconds or more show up here." } },
                        _ => rsx! {},
                    }
                }
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
                                class: if picks.is_favourite(&song.id) { "card-btn fav-btn on" } else { "card-btn fav-btn" },
                                title: if picks.is_favourite(&song.id) { "Remove from favourites" } else { "Add to favourites" },
                                aria_label: "Favourite",
                                aria_pressed: "{picks.is_favourite(&song.id)}",
                                onclick: {
                                    let id = song.id.clone();
                                    move |_| on_toggle_favourite.call(id.clone())
                                },
                                if picks.is_favourite(&song.id) { "★" } else { "☆" }
                            }
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
