use dioxus::prelude::*;

use crate::mic::{Mic, MicError};
use crate::pitch::{Mpm, MpmConfig, NoteReading};
use crate::score::{TakeResult, TuningScorer, TuningSummary, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS};
use crate::types::Song;

#[derive(Debug, Clone, PartialEq)]
enum MicState {
    Off,
    Starting,
    Listening,
    Failed(MicError),
}

/// Live pitch of the singer's mic (note name + cents) and a tuning score for the current take.
/// `take` changes on every song start or replay, which restarts the score and reports the finished
/// take through `on_take_end` (only if the mic judged at least one held note).
#[component]
pub fn PitchMeter(take: f64, song: Song, on_take_end: EventHandler<TakeResult>) -> Element {
    // Owns the device; dropping the session (toggle off or unmount) releases the mic
    let mut session = use_signal(|| None::<Mic>);
    let mut state = use_signal(|| MicState::Off);
    let mut reading = use_signal(|| None::<NoteReading>);
    let mut summary = use_signal(TuningSummary::default);
    let mut current_take = use_signal(|| take);
    // Song of the take being scored: guide edits change `song` without starting a new take
    let song_meta = (song.id, song.title, song.artist);
    let mut take_song = use_signal(|| song_meta.clone());

    use_effect(use_reactive!(|take, song_meta| {
        if *current_take.peek() != take {
            let (id, title, artist) = &*take_song.peek();
            if let Some(result) = TakeResult::new(id, title, artist, *summary.peek()) {
                on_take_end.call(result);
            }
            current_take.set(take);
            summary.set(TuningSummary::default());
        }
        take_song.set(song_meta);
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
/// A tap target (`details`), not a hover tooltip: phones and tablets never show `title` text.
#[component]
fn TuningBadge(summary: TuningSummary) -> Element {
    let notes = summary.notes;
    let average = summary.mean_abs_cents.map_or(String::new(), |c| format!(", on average {c:.0}¢ off the nearest semitone"));
    let pending = match notes < MIN_SCORED_NOTES {
        true => format!(" (a score needs {MIN_SCORED_NOTES})"),
        false => String::new(),
    };
    let progress = format!("This song so far: {notes} held notes{average}{pending}.");
    rsx! {
        details { class: "tuning-score",
            summary { title: "What do these numbers mean?",
                span { class: "tuning-label", "Tuning" }
                match summary.score() {
                    Some(score) => rsx! { span { class: "tuning-value", "{score}" } },
                    None => rsx! { span { class: "tuning-value idle", "—" } },
                }
                span { class: "tuning-help", aria_hidden: "true", "?" }
            }
            div { class: "tuning-explain",
                p { strong { "Note (e.g. A4 +12¢): " } "the pitch you are singing right now. ¢ = cents: + is sharp, − is flat; 100¢ is one semitone." }
                p { strong { "Tuning 0-100: " } "how exactly your held notes (sung steadily for about ⅕ s) land on a semitone. 100 = within {PERFECT_CENTS}¢ on average, 0 = {RANDOM_CENTS}¢ or more off. It appears after {MIN_SCORED_NOTES} held notes and restarts with each song or Replay." }
                p { "{progress}" }
                p { class: "tuning-caveat", "It does not know the song's melody, so it cannot tell whether they are the right notes. Loud speakers leaking into the mic can also move it." }
            }
        }
    }
}
