use dioxus::prelude::*;

/// Booth volume for the karaoke and original-vocal players (0-100, saved in settings). ↑ / ↓ on the keyboard,
/// a controller's D-pad, and phones allowed to control playback change it too.
#[component]
pub fn VolumeControl(volume: u32, on_volume: EventHandler<u32>) -> Element {
    let icon = match volume {
        0 => "🔇",
        1..=40 => "🔉",
        _ => "🔊",
    };
    rsx! {
        label { class: "volume-control", title: "Volume (↑ / ↓ on the keyboard)",
            span { class: "volume-icon", aria_hidden: "true", "{icon}" }
            input {
                r#type: "range",
                id: "volume",
                name: "volume",
                class: "volume-slider",
                min: "0",
                max: "100",
                step: "5",
                value: "{volume}",
                aria_label: "Volume",
                aria_valuetext: "{volume}%",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u32>() {
                        on_volume.call(v.min(100));
                    }
                },
            }
            span { class: "volume-value", "{volume}" }
        }
    }
}
