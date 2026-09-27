use dioxus::prelude::*;

use crate::browser;
use crate::storage::{self, TEMPO_KEY};
use crate::sync::{self, SyncCommand};
use crate::tempo::{self, BeatGrids, CountIn, MIN_TAPS};

/// Longest wait for the video to start playing after the count-in seek before giving up on the clicks.
const START_TIMEOUT_MS: i32 = 5_000;
const POLL_MS: i32 = 25;

/// Tap tempo + count-in (musician mode). Tap along with the beat a few times and the song's beat grid is saved on
/// this device; Count-in then jumps a few beats before the loop's A (or where the song is) and clicks them in.
#[component]
pub fn TempoTools(
    song_id: String,
    /// Current karaoke second, used when the sync core cannot give a finer one.
    fallback_sec: u64,
    /// Where the practice part starts: the loop's A, if one is set.
    part_start: Option<f64>,
) -> Element {
    let mut grids = use_signal(|| storage::load::<BeatGrids>(TEMPO_KEY).map(tempo::sanitize).unwrap_or_default());
    use_effect(move || storage::save(TEMPO_KEY, &*grids.read()));
    let mut taps = use_signal(Vec::<f64>::new);
    let mut counting = use_signal(|| false);
    let id = song_id.clone();
    use_effect(use_reactive!(|song_id| {
        let _ = song_id;
        taps.write().clear();
    }));
    let now = move || sync::karaoke_time().unwrap_or(fallback_sec as f64);
    let grid = grids.read().get(&id).copied();

    let tap = {
        let id = id.clone();
        move |_| {
            let mut series = taps.write();
            tempo::add_tap(&mut series, now());
            if let Some(fitted) = tempo::fit(&series) {
                grids.write().insert(id.clone(), fitted);
            }
        }
    };
    let count_in = move |_| {
        let Some(grid) = grid else { return };
        browser::prepare_clicks();
        let plan = grid.count_in(part_start.unwrap_or_else(now));
        counting.set(true);
        spawn(async move {
            play_count_in(&plan).await;
            counting.set(false);
        });
    };

    let tapped = taps.read().len();
    let label = match (grid, tapped) {
        (_, n) if (1..MIN_TAPS).contains(&n) => format!("Tap {} more", MIN_TAPS - n),
        (Some(g), _) => format!("♩ {:.0}", g.bpm()),
        (None, _) => "Tap the beat".to_string(),
    };

    rsx! {
        span { class: "tempo-tools",
            button {
                class: "ctrl-btn practice-btn tempo-tap",
                title: "Tap along with the beat (4 taps or more) to set this song's tempo; saved on this device",
                onclick: tap,
                "Tap"
            }
            span { class: "practice-label tempo-label", role: "status", "{label}" }
            button {
                class: if counting() { "ctrl-btn practice-btn guide-active" } else { "ctrl-btn practice-btn" },
                title: "Jump 4 beats before the loop's A (or here) and click them in",
                disabled: grid.is_none() || counting(),
                onclick: count_in,
                "Count-in"
            }
        }
    }
}

/// Seek to the count-in start (resuming if paused), wait until the video is playing, then click the beats in.
async fn play_count_in(plan: &CountIn) {
    SyncCommand::Restart(plan.start).run();
    let mut waited = 0;
    // Playing again: the clock has moved on from the seek target
    while sync::karaoke_time().is_none_or(|t| t < plan.start + 0.05 || t > plan.land) {
        if waited >= START_TIMEOUT_MS {
            return;
        }
        browser::sleep_ms(POLL_MS).await;
        waited += POLL_MS;
    }
    let Some(t) = sync::karaoke_time() else { return };
    let delays: Vec<f64> = plan.clicks.iter().map(|click| click - t).collect();
    browser::clicks_in(&delays);
    // Stay "counting" until the part begins, so the button cannot stack a second count-in on top
    let until_land = ((plan.land - t) * 1000.0).max(0.0) as i32;
    browser::sleep_ms(until_land).await;
}
