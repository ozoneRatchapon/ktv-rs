use dioxus::prelude::*;
use crate::catalog::{CustomCodesExhausted, CUSTOM_CODES};
use crate::types::Song;
use crate::youtube::parse_video_id;

/// Outcome shown above the form after a submit.
#[derive(Clone, Debug, PartialEq)]
enum Feedback {
    Added { code: String },
    Error(String),
}

#[component]
pub fn CustomAdd(
    /// (song, play_now) → the stored song with its assigned keypad code
    on_add_song: Callback<(Song, bool), Result<Song, CustomCodesExhausted>>,
) -> Element {
    let mut url_or_id = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut artist = use_signal(String::new);
    let mut intro_skip = use_signal(|| 13u32);
    let mut feedback = use_signal(|| None::<Feedback>);

    let submit_action = move |play_now: bool| {
        let raw_url = url_or_id();
        let maybe_id = parse_video_id(&raw_url);

        match maybe_id {
            Some(vid) => {
                let song_title = if title().trim().is_empty() {
                    format!("Custom YouTube Video ({vid})")
                } else {
                    title().trim().to_string()
                };

                let song_artist = if artist().trim().is_empty() {
                    "YouTube".to_string()
                } else {
                    artist().trim().to_string()
                };

                let new_song = Song {
                    id: format!("custom_{vid}"),
                    code: String::new(), // assigned on insert (catalog::upsert_custom)
                    title: song_title,
                    artist: song_artist,
                    youtube_id: vid,
                    guide: None,
                    duration_secs: 240,
                    intro_skip_secs: intro_skip(),
                    category: "Custom".to_string(),
                    channel: "YouTube Direct".to_string(),
                    is_favorite: false,
                };

                match on_add_song.call((new_song, play_now)) {
                    Ok(stored) => {
                        feedback.set(Some(Feedback::Added { code: stored.code }));
                        url_or_id.set(String::new());
                        title.set(String::new());
                        artist.set(String::new());
                    }
                    // Keep the form filled so nothing typed is lost
                    Err(CustomCodesExhausted) => {
                        let (first, last) = (CUSTOM_CODES.start(), CUSTOM_CODES.end());
                        feedback.set(Some(Feedback::Error(format!(
                            "Could not add the song: all custom song codes ({first}-{last}) are in use"
                        ))));
                    }
                }
            }
            None => {
                feedback.set(Some(Feedback::Error("Enter a valid YouTube URL or 11-character video ID".to_string())));
            }
        }
    };

    rsx! {
        div { class: "custom-add-container",
            div { class: "custom-add-card",
                div { class: "card-heading",
                    div {
                        h3 { "Add Custom YouTube Song" }
                        p { "Enter any YouTube video URL or ID with optional intro skip offset" }
                    }
                }

                match feedback() {
                    Some(Feedback::Added { code }) => rsx! {
                        div { class: "form-feedback", role: "status", "Song added! Keypad code {code}" }
                    },
                    Some(Feedback::Error(msg)) => rsx! {
                        div { class: "form-feedback error", role: "alert", "{msg}" }
                    },
                    None => rsx! {},
                }

                div { class: "form-group",
                    label { r#for: "custom_url", "YouTube URL or Video ID *" }
                    input {
                        id: "custom_url",
                        name: "custom_url",
                        class: "form-input",
                        r#type: "text",
                        placeholder: "e.g. https://www.youtube.com/watch?v=9aCUDQ8SPcA or 9aCUDQ8SPcA",
                        value: "{url_or_id()}",
                        oninput: move |evt| url_or_id.set(evt.value()),
                    }
                }

                div { class: "form-row",
                    div { class: "form-group flex-1",
                        label { r#for: "custom_title", "Song Title" }
                        input {
                            id: "custom_title",
                            name: "custom_title",
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Title",
                            value: "{title()}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div { class: "form-group flex-1",
                        label { r#for: "custom_artist", "Artist" }
                        input {
                            id: "custom_artist",
                            name: "custom_artist",
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Artist",
                            value: "{artist()}",
                            oninput: move |evt| artist.set(evt.value()),
                        }
                    }
                }

                div { class: "form-group",
                    label { r#for: "custom_intro_skip", "Intro Skip Offset (Seconds)" }
                    div { class: "intro-preset-row",
                        button {
                            class: if intro_skip() == 0 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(0),
                            "0s"
                        }
                        button {
                            class: if intro_skip() == 10 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(10),
                            "10s"
                        }
                        button {
                            class: if intro_skip() == 13 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(13),
                            "13s"
                        }
                        button {
                            class: if intro_skip() == 15 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(15),
                            "15s"
                        }
                    }
                    input {
                        id: "custom_intro_skip",
                        name: "custom_intro_skip",
                        class: "form-input mt-2",
                        r#type: "number",
                        min: "0",
                        max: "60",
                        value: "{intro_skip()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<u32>() {
                                intro_skip.set(val);
                            }
                        },
                    }
                }

                div { class: "form-actions-row",
                    button {
                        class: "submit-btn primary-btn",
                        onclick: {
                            let mut action = submit_action;
                            move |_| action(true)
                        },
                        span { "Play Now" }
                    }
                    button {
                        class: "submit-btn secondary-btn",
                        onclick: {
                            let mut action = submit_action;
                            move |_| action(false)
                        },
                        span { "Add to Queue" }
                    }
                }
            }
        }
    }
}
