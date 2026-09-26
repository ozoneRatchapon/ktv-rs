use dioxus::prelude::*;

use crate::mic::{Mic, MicError};
use crate::pitch::{Mpm, MpmConfig, NoteReading};
use crate::score::{TuningScorer, TuningSummary};

#[derive(Debug, Clone, PartialEq)]
enum MicState {
    Off,
    Starting,
    Listening,
    Failed(MicError),
}

/// Live pitch of the singer's mic (note name + cents) and a tuning score for the current take.
/// `take` changes on every song start or replay, which restarts the score.
#[component]
pub fn PitchMeter(take: f64) -> Element {
    // Owns the device; dropping the session (toggle off or unmount) releases the mic
    let mut session = use_signal(|| None::<Mic>);
    let mut state = use_signal(|| MicState::Off);
    let mut reading = use_signal(|| None::<NoteReading>);
    let mut summary = use_signal(TuningSummary::default);
    let mut current_take = use_signal(|| take);

    use_effect(use_reactive!(|take| {
        current_take.set(take);
        summary.set(TuningSummary::default());
    }));

    let toggle = move |_| {
        if session.write().take().is_some() {
            state.set(MicState::Off);
            reading.set(None);
            return;
        }
        state.set(MicState::Starting);
        summary.set(TuningSummary::default());
        spawn(async move {
            let started = Mic::start(move |sample_rate| {
                let mut detector = Mpm::new(MpmConfig::singing(sample_rate));
                let mut scorer = TuningScorer::new();
                let mut scored_take = *current_take.peek();
                move |frame: &[f32]| {
                    let estimate = detector.detect(frame);
                    if *current_take.peek() != scored_take {
                        scored_take = *current_take.peek();
                        scorer = TuningScorer::new();
                    }
                    // ~47 frames/s: only re-render when the shown note changes or a held note is judged
                    if scorer.push(estimate.map(|est| est.midi())).is_some() {
                        summary.set(scorer.summary());
                    }
                    let next = estimate.map(|est| est.reading());
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
                TuningBadge { summary: summary() }
            }
        }
    }
}

/// Reference-free score: says what it measures, and what it cannot know.
#[component]
fn TuningBadge(summary: TuningSummary) -> Element {
    let detail = match summary.mean_abs_cents {
        Some(cents) => format!("{} held notes, on average {cents:.0}¢ off the nearest semitone", summary.notes),
        None => "Hold a few notes to get a score".to_string(),
    };
    let title = format!(
        "Tuning: how close your held notes sit to exact semitones ({detail}). \
         Without the song's melody it cannot tell whether they are the right notes; \
         loud speakers leaking into the mic can also raise it."
    );
    rsx! {
        span { class: "tuning-score", title: "{title}",
            span { class: "tuning-label", "Tuning" }
            match summary.score() {
                Some(score) => rsx! { span { class: "tuning-value", "{score}" } },
                None => rsx! { span { class: "tuning-value idle", "—" } },
            }
        }
    }
}
