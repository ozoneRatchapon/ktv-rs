use std::rc::Rc;

use dioxus::prelude::*;

use crate::mic::{Mic, MicChannels, MicError, HOP_SIZE};
use crate::pitch::{rms, Mpm, MpmConfig, NoiseGate, NoteReading};
use crate::score::{
    stored_melody, LaneView, LaneWindow, Melody, MelodyScorer, MelodySummary, NoteLane, StoredMelodies, TakeResult,
    TuningScorer, TuningSummary, FULL_COVERAGE, MIN_PHRASE_NOTES, MIN_SCORED_NOTES, PERFECT_CENTS, RANDOM_CENTS,
};
use crate::storage::{self, MELODIES_KEY};
use crate::sync;
use crate::types::Song;

/// Seconds of singing the note lane shows.
const LANE_SECONDS: f64 = 8.0;
/// With a melody, the lane also shows this much of what is coming (the tune's next notes).
const LANE_AHEAD_SECONDS: f64 = 2.0;
/// With a melody the targets scroll, so the lane is redrawn every few frames (~10 per second), not only on changes.
const SCROLL_EVERY_FRAMES: u32 = 5;
/// Voices one mic session can carry (a duet on a stereo receiver).
const MAX_VOICES: usize = 2;

#[derive(Debug, Clone, PartialEq)]
enum MicState {
    Off,
    Starting,
    Listening,
    Failed(MicError),
}

/// What one singer's row shows. Signals, so the mic callback updates only what changed (they compare by identity).
#[derive(Clone, Copy, PartialEq)]
struct VoiceView {
    reading: Signal<Option<NoteReading>>,
    /// First second after the mic opens: measuring the room for this voice's noise gate.
    room_check: Signal<bool>,
    summary: Signal<TuningSummary>,
    melody_summary: Signal<MelodySummary>,
    lane: Signal<LaneView>,
    phrase: Signal<Option<u8>>,
}

impl VoiceView {
    fn use_new() -> Self {
        Self {
            reading: use_signal(|| None),
            room_check: use_signal(|| false),
            summary: use_signal(TuningSummary::default),
            melody_summary: use_signal(MelodySummary::default),
            lane: use_signal(LaneView::default),
            phrase: use_signal(|| None),
        }
    }

    /// New take (or mic restart): scores and lane start empty.
    fn reset(mut self) {
        self.summary.set(TuningSummary::default());
        self.melody_summary.set(MelodySummary::default());
        self.lane.set(LaneView::default());
        self.phrase.set(None);
    }
}

/// One singer's analysis, owned by the mic callback (no signals: it runs ~47 times a second per voice).
struct Voice {
    detector: Mpm,
    gate: NoiseGate,
    scorer: TuningScorer,
    notes: NoteLane,
    tune: MelodyScorer,
    frame_secs: f64,
    frames_since_draw: u32,
}

impl Voice {
    fn new(sample_rate: f32) -> Self {
        let frame_secs = f64::from(HOP_SIZE) / f64::from(sample_rate);
        Self {
            detector: Mpm::new(MpmConfig::singing(sample_rate)),
            gate: NoiseGate::new(),
            scorer: TuningScorer::new(),
            notes: NoteLane::new(frame_secs),
            tune: MelodyScorer::new(),
            frame_secs,
            frames_since_draw: 0,
        }
    }

    /// New take: the noise gate keeps what it learned about the room.
    fn new_take(&mut self) {
        self.scorer = TuningScorer::new();
        self.notes = NoteLane::new(self.frame_secs);
        self.tune = MelodyScorer::new();
    }

    /// Analyse one frame heard at song time `t`. Re-renders only when the shown note changes, a held note is judged,
    /// a phrase ends, or (with a melody) the targets have scrolled a little.
    fn frame(&mut self, frame: &[f32], t: f64, melody: Option<&Melody>, mut view: VoiceView) {
        let singing = self.gate.pass(rms(frame));
        if *view.room_check.peek() != self.gate.is_checking() {
            view.room_check.set(self.gate.is_checking());
        }
        let estimate = if singing { self.detector.detect(frame) } else { None };
        let midi = estimate.map(|est| est.midi());
        if let Some(m) = melody {
            if self.tune.push(m, t, midi) && *view.melody_summary.peek() != self.tune.summary() {
                view.melody_summary.set(self.tune.summary());
            }
        }
        let judged = self.scorer.push(midi);
        if judged.is_some() {
            view.summary.set(self.scorer.summary());
        }
        self.frames_since_draw += 1;
        let scroll = melody.is_some() && self.frames_since_draw >= SCROLL_EVERY_FRAMES;
        if self.notes.push(t, estimate.is_some(), judged) || scroll {
            self.frames_since_draw = 0;
            let window = match melody {
                Some(_) => LaneWindow { past: LANE_SECONDS - LANE_AHEAD_SECONDS, ahead: LANE_AHEAD_SECONDS },
                None => LaneWindow { past: LANE_SECONDS, ahead: 0.0 },
            };
            view.lane.set(self.notes.view(window, melody));
            view.phrase.set(self.notes.phrase_score());
        }
        let next = estimate.map(|est| est.reading());
        if *view.reading.peek() != next {
            view.reading.set(next);
        }
    }
}

/// Live pitch of the singer's mic (note name + cents), note lane and scores for the current take; with `duet`,
/// one row per singer on a stereo receiver (left = 1, right = 2). `take` changes on every song start or replay,
/// which restarts the scores and reports each voice's finished take through `on_take_end` (a voice that judged
/// no held note reports nothing).
#[component]
pub fn PitchMeter(take: f64, song: Song, duet: bool, on_take_end: EventHandler<TakeResult>) -> Element {
    // Owns the device; dropping the session (toggle off or unmount) releases the mic
    let mut session = use_signal(|| None::<Mic>);
    let mut state = use_signal(|| MicState::Off);
    // Channels of the running session (duet is read when the mic starts)
    let mut active = use_signal(MicChannels::default);
    let views: [VoiceView; MAX_VOICES] = [VoiceView::use_new(), VoiceView::use_new()];
    let mut current_take = use_signal(|| take);
    // The song's melody, if this device has one (none ship; see plan 002 item 7): enables the melody score
    let song_id = song.id.clone();
    let melody = use_memo(use_reactive((&song_id,), |(id,)| {
        storage::load::<StoredMelodies>(MELODIES_KEY).and_then(|store| stored_melody(&store, &id)).map(Rc::new)
    }));
    // Song of the take being scored: guide edits change `song` without starting a new take
    let song_meta = (song.id, song.title, song.artist);
    let mut take_song = use_signal(|| song_meta.clone());

    use_effect(use_reactive!(|take, song_meta| {
        if *current_take.peek() != take {
            let (id, title, artist) = &*take_song.peek();
            let ended_at = js_sys::Date::now();
            let duet = *active.peek() == MicChannels::Stereo;
            for (i, view) in views.iter().enumerate().take(active.peek().count()) {
                let Some(result) = TakeResult::new(id, title, artist, *view.summary.peek(), ended_at) else { continue };
                let part = duet.then_some(i as u8 + 1);
                on_take_end.call(result.with_melody_score(view.melody_summary.peek().score()).with_part(part));
            }
            current_take.set(take);
            views.iter().for_each(|v| v.reset());
        }
        take_song.set(song_meta);
    }));

    // Switching duet on or off needs the device opened again: stop, and the next tap starts the new mode
    use_effect(use_reactive!(|duet| {
        let wanted = if duet { MicChannels::Stereo } else { MicChannels::Mono };
        if session.peek().is_some() && *active.peek() != wanted {
            session.set(None);
            state.set(MicState::Off);
        }
    }));

    let toggle = move |_| {
        if session.write().take().is_some() {
            state.set(MicState::Off);
            for mut view in views {
                view.reading.set(None);
            }
            return;
        }
        let channels = if duet { MicChannels::Stereo } else { MicChannels::Mono };
        active.set(channels);
        state.set(MicState::Starting);
        for mut view in views {
            view.reset();
            view.room_check.set(true);
        }
        spawn(async move {
            let started = Mic::start(channels, move |sample_rate| {
                let mut voices: Vec<Voice> = (0..channels.count()).map(|_| Voice::new(sample_rate)).collect();
                let frame_secs = f64::from(HOP_SIZE) / f64::from(sample_rate);
                // Without a video clock (never in the app, but a stalled bridge must not stop the lane), frames count time
                let mut frame_clock = 0.0f64;
                let mut t = 0.0f64;
                let mut scored_take = *current_take.peek();
                move |channel: usize, frame: &[f32]| {
                    // Channel 0 arrives first in each message: read the clock and check for a new take once per frame
                    if channel == 0 {
                        if *current_take.peek() != scored_take {
                            scored_take = *current_take.peek();
                            voices.iter_mut().for_each(Voice::new_take);
                        }
                        frame_clock += frame_secs;
                        t = sync::karaoke_time().unwrap_or(frame_clock);
                    }
                    let (Some(voice), Some(&view)) = (voices.get_mut(channel), views.get(channel)) else { return };
                    let melody = melody.peek();
                    voice.frame(frame, t, melody.as_deref(), view);
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
    let duet_on = active() == MicChannels::Stereo;

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
                for (i, view) in views.iter().enumerate().take(active().count()) {
                    div { key: "{i}", class: if duet_on { "voice-row duet" } else { "voice-row" },
                        if duet_on {
                            span { class: "voice-tag", title: if i == 0 { "Singer 1: left channel" } else { "Singer 2: right channel" }, "{i + 1}" }
                        }
                        VoiceRow { view: *view, has_melody: melody().is_some() }
                    }
                }
            }
        }
    }
}

/// Readout, lane and scores for one voice.
#[component]
fn VoiceRow(view: VoiceView, has_melody: bool) -> Element {
    let reading = (view.reading)();
    rsx! {
        span {
            class: "pitch-readout",
            title: "Detected pitch of your voice (not a score)",
            aria_live: "polite",
            match ((view.room_check)(), reading) {
                (true, _) => rsx! {
                    span { class: "pitch-note idle", title: "Stay quiet for a second: measuring the room so its noise is ignored", "Room check…" }
                },
                (false, Some(note)) => rsx! {
                    span { class: "pitch-note", "{note.name()}" }
                    span { class: "pitch-cents", "{note.cents:+}¢" }
                },
                (false, None) => rsx! { span { class: "pitch-note idle", "—" } },
            }
        }
        NoteLaneView { view: (view.lane)(), phrase: (view.phrase)() }
        if has_melody {
            MelodyBadge { summary: (view.melody_summary)() }
        }
        TuningBadge { summary: (view.summary)() }
    }
}

/// The singer's held notes over the last few seconds on a semitone grid: a bar on a line is in tune, a bar
/// between lines is off. Colour repeats it for a quick glance. Then the last phrase's score.
#[component]
fn NoteLaneView(view: LaneView, phrase: Option<u8>) -> Element {
    // One row per semitone with half a row of margin above and below, y grows downward
    let rows = (view.high - view.low + 1).max(1) as f32;
    let y = move |midi: f32| (view.high as f32 + 0.5 - midi) / rows * 100.0;
    let row = 100.0 / rows;
    let label = match phrase {
        Some(score) => format!("Last phrase {score}"),
        None => "Your held notes".to_string(),
    };
    rsx! {
        div { class: "note-lane", title: "Your held notes, last {LANE_SECONDS:.0} s: on a line = on a semitone. Phrase score after each breath (needs {MIN_PHRASE_NOTES} held notes).",
            svg {
                class: "note-lane-plot",
                view_box: "0 0 100 100",
                preserve_aspect_ratio: "none",
                role: "img",
                "aria-label": "{label}",
                for midi in view.low..=view.high {
                    line { key: "g{midi}", class: "lane-grid", x1: "0", x2: "100", y1: "{y(midi as f32)}", y2: "{y(midi as f32)}" }
                }
                // The tune's notes (when the song has a melody) behind the singer's, moved to the singer's octave
                for (i, target) in view.targets.iter().enumerate() {
                    rect {
                        key: "t{i}",
                        class: "lane-target",
                        x: "{target.x0 * 100.0}",
                        width: "{((target.x1 - target.x0) * 100.0).max(1.0)}",
                        y: "{y(target.midi) - row * 0.45}",
                        height: "{row * 0.9}",
                        rx: "1",
                    }
                }
                if let Some(now_x) = view.now_x {
                    line { class: "lane-now", x1: "{now_x * 100.0}", x2: "{now_x * 100.0}", y1: "0", y2: "100" }
                }
                for (i, bar) in view.bars.iter().enumerate() {
                    rect {
                        key: "{i}",
                        class: match bar.cents.abs() {
                            c if c <= PERFECT_CENTS => "lane-note good",
                            c if c < RANDOM_CENTS => "lane-note near",
                            _ => "lane-note off",
                        },
                        x: "{bar.x0 * 100.0}",
                        width: "{((bar.x1 - bar.x0) * 100.0).max(1.0)}",
                        y: "{y(bar.midi) - row * 0.3}",
                        height: "{row * 0.6}",
                        rx: "1",
                    }
                }
            }
            span { class: "note-lane-phrase",
                span { class: "tuning-label", "Phrase" }
                match phrase {
                    Some(score) => rsx! { span { class: "tuning-value", "{score}" } },
                    None => rsx! { span { class: "tuning-value idle", "—" } },
                }
            }
        }
    }
}

/// Melody score, shown only when this device has the song's melody. Same tap-to-explain form as the Tuning badge.
#[component]
fn MelodyBadge(summary: MelodySummary) -> Element {
    let heard = summary.target_frames;
    let hit = summary.hit_frames;
    let pending = match summary.score() {
        Some(_) => String::new(),
        None => " A score appears after about 2 s of the tune.".to_string(),
    };
    rsx! {
        details { class: "tuning-score melody-score",
            summary { title: "What does this number mean?",
                span { class: "tuning-label", "Melody" }
                match summary.score() {
                    Some(score) => rsx! { span { class: "tuning-value", "{score}" } },
                    None => rsx! { span { class: "tuning-value idle", "—" } },
                }
                span { class: "tuning-help", aria_hidden: "true", "?" }
            }
            div { class: "tuning-explain",
                p { strong { "Melody 0-100: " } "how much of the tune you sang on the right note. Any octave counts, so low and high voices score the same. 100 = the right note for {FULL_COVERAGE * 100.0:.0}% of the tune's notes (breaths and consonants make all of it impossible)." }
                p { "This song so far: right note in {hit} of {heard} moments of the tune.{pending}" }
                p { class: "tuning-caveat", "The grey bars in the lane are the tune, moved to your octave. The melody comes from data saved on this device for this song." }
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
                p { class: "tuning-caveat", "When the mic turns on it listens to the room for a second; anything not clearly louder than the room (about twice its level) is ignored. Turn the mic off and on to check again." }
            }
        }
    }
}
