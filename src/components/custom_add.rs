use dioxus::prelude::*;
use crate::types::Song;

#[component]
pub fn CustomAdd(
    on_add_song: EventHandler<(Song, bool)>, // (song, play_now)
) -> Element {
    let mut url_or_id = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut artist = use_signal(String::new);
    let mut intro_skip = use_signal(|| 13u32);
    let mut feedback = use_signal(|| None::<String>);

    let parse_youtube_id = |input: &str| -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(pos) = trimmed.find("v=") {
            let rest = &trimmed[pos + 2..];
            let id = rest.split('&').next().unwrap_or(rest);
            return Some(id.to_string());
        }

        if let Some(pos) = trimmed.find("youtu.be/") {
            let rest = &trimmed[pos + 9..];
            let id = rest.split('?').next().unwrap_or(rest);
            return Some(id.to_string());
        }

        Some(trimmed.to_string())
    };

    let submit_action = move |play_now: bool| {
        let raw_url = url_or_id();
        let maybe_id = parse_youtube_id(&raw_url);

        match maybe_id {
            Some(vid) if !vid.is_empty() => {
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
                    code: "99999".to_string(),
                    title: song_title,
                    artist: song_artist,
                    youtube_id: vid,
                    guide_video_id: None,
                    duration_secs: 240,
                    intro_skip_secs: intro_skip(),
                    category: "Custom".to_string(),
                    channel: "YouTube Direct".to_string(),
                    is_favorite: false,
                };

                on_add_song.call((new_song, play_now));
                feedback.set(Some("เพิ่มเพลงสำเร็จเรียบร้อยแล้ว!".to_string()));
                url_or_id.set(String::new());
                title.set(String::new());
                artist.set(String::new());
            }
            _ => {
                feedback.set(Some("กรุณากรอก YouTube URL หรือ Video ID ให้ถูกต้อง".to_string()));
            }
        }
    };

    rsx! {
        div { class: "custom-add-container",
            div { class: "custom-add-card",
                div { class: "card-heading",
                    span { class: "heading-icon", "📺" }
                    div {
                        h3 { "เพิ่มเพลงจาก YouTube อิสระ" }
                        p { "วางลิงก์วิดีโอคาราโอเกะใดก็ได้จาก YouTube พร้อมตั้งค่าเวลาตัดอินโทร" }
                    }
                }

                if let Some(fb) = feedback() {
                    div { class: "form-feedback", "{fb}" }
                }

                div { class: "form-group",
                    label { "YouTube URL หรือ Video ID *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "e.g. https://www.youtube.com/watch?v=inGSjouS77g หรือ inGSjouS77g",
                        value: "{url_or_id()}",
                        oninput: move |evt| url_or_id.set(evt.value()),
                    }
                }

                div { class: "form-row",
                    div { class: "form-group flex-1",
                        label { "ชื่อเพลง (Title)" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "ชื่อเพลง",
                            value: "{title()}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div { class: "form-group flex-1",
                        label { "ศิลปิน (Artist)" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "ชื่อศิลปิน",
                            value: "{artist()}",
                            oninput: move |evt| artist.set(evt.value()),
                        }
                    }
                }

                div { class: "form-group",
                    label { "ระยะเวลาข้าม Intro แพลตฟอร์ม (วินาที)" }
                    div { class: "intro-preset-row",
                        button {
                            class: if intro_skip() == 0 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(0),
                            "0s (ไม่ข้าม)"
                        }
                        button {
                            class: if intro_skip() == 10 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(10),
                            "10s (สั้น)"
                        }
                        button {
                            class: if intro_skip() == 13 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(13),
                            "13s (GMM มาตรฐาน)"
                        }
                        button {
                            class: if intro_skip() == 15 { "preset-btn active" } else { "preset-btn" },
                            onclick: move |_| intro_skip.set(15),
                            "15s"
                        }
                    }
                    input {
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
                        class: "submit-btn primary-glow",
                        onclick: {
                            let mut action = submit_action;
                            move |_| action(true)
                        },
                        span { "▶️ ร้องทันที (Play Now)" }
                    }
                    button {
                        class: "submit-btn secondary-btn",
                        onclick: {
                            let mut action = submit_action;
                            move |_| action(false)
                        },
                        span { "➕ เพิ่มเข้าคิว (Add Queue)" }
                    }
                }
            }
        }
    }
}
