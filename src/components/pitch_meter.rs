use dioxus::prelude::*;

use crate::mic::{Mic, MicError};
use crate::pitch::{Mpm, MpmConfig, NoteReading};

#[derive(Debug, Clone, PartialEq)]
enum MicState {
    Off,
    Starting,
    Listening,
    Failed(MicError),
}

/// Live pitch of the singer's mic (note name + cents). Display only: no scoring yet.
#[component]
pub fn PitchMeter() -> Element {
    // Owns the device; dropping the session (toggle off or unmount) releases the mic
    let mut session = use_signal(|| None::<Mic>);
    let mut state = use_signal(|| MicState::Off);
    let mut reading = use_signal(|| None::<NoteReading>);

    let toggle = move |_| {
        if session.write().take().is_some() {
            state.set(MicState::Off);
            reading.set(None);
            return;
        }
        state.set(MicState::Starting);
        spawn(async move {
            let started = Mic::start(move |sample_rate| {
                let mut detector = Mpm::new(MpmConfig::singing(sample_rate));
                move |frame: &[f32]| {
                    let next = detector.detect(frame).map(|est| est.reading());
                    // ~47 frames/s: only re-render when the shown note actually changes
                    if *reading.peek() != next {
                        reading.set(next);
                    }
                }
            })
            .await;
            match started {
                Ok(mic) => {
                    session.set(Some(mic));
                    state.set(MicState::Listening);
                }
                Err(err) => state.set(MicState::Failed(err)),
            }
        });
    };

    let (label, class, title) = match state() {
        MicState::Off => ("Mic: Off".to_string(), "ctrl-btn action-btn", "Show the pitch you are singing (mic stays on this device)".to_string()),
        MicState::Starting => ("Mic: …".to_string(), "ctrl-btn action-btn", "Waiting for microphone permission".to_string()),
        MicState::Listening => ("Mic: On".to_string(), "ctrl-btn action-btn guide-active", "Stop listening".to_string()),
        MicState::Failed(err) => ("Mic: Error".to_string(), "ctrl-btn action-btn", format!("{err} (click to retry)")),
    };

    rsx! {
        div { class: "pitch-meter",
            button {
                class,
                title: "{title}",
                disabled: state() == MicState::Starting,
                onclick: toggle,
                span { "{label}" }
            }
            if state() == MicState::Listening {
                span {
                    class: "pitch-readout",
                    title: "Detected pitch of your voice (not a score)",
                    aria_live: "polite",
                    match reading() {
                        Some(note) => rsx! {
                            span { class: "pitch-note", "{note.name()}" }
                            span { class: "pitch-cents", "{note.cents:+}¢" }
                        },
                        None => rsx! { span { class: "pitch-note idle", "—" } },
                    }
                }
            }
        }
    }
}
