use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::HashSet;
use std::rc::Rc;

use app::booth::{Booth, Placement, Requester};
use app::catalog;
use app::components;
use app::keys::{self, KeyAction};
use app::library;
use app::recommendation;
use app::picks::Picks;
use app::score::{self, TakeResult};
use app::storage::{self, Session, GUIDES_KEY, PICKS_KEY, SCORES_KEY, SESSION_KEY, SETTINGS_KEY};
use app::sync::SyncCommand;
use app::timing::{self, GuideOverrides, SavedTiming};
use app::types;

use catalog::builtin_catalog;
use components::{
    catalog_view::CatalogView,
    custom_add::CustomAdd,
    header::Header,
    player::Player,
    queue_view::QueueView,
    quick_search::QuickSearch,
    remote::Remote,
    settings::Settings,
    score_card::ScoreCard,
    shortcuts::ShortcutHelp,
    up_next::UpNext,
};
use recommendation::{SleepTimeAnticipator, SongTelemetry};
use types::{AppSettings, GuideTrack, KtvTab, QueueItem, Song};

// In the static <head> at build time: the stylesheet loads alongside the wasm instead of after it has run
const _: Asset = asset!("/assets/main.css", AssetOptions::css().with_static_head(true));
// Classic scripts in the static <head>: they run before the wasm, so Rust calls them directly (no eval, see js_bridge)
const _: Asset = asset!("/assets/ktv_sync.js", AssetOptions::js().with_static_head(true));
const _: Asset = asset!("/assets/ktv_keys.js", AssetOptions::js().with_static_head(true));
// The full songbook, fetched after the first paint (content-hashed, so cached for good)
const LIBRARY_JSON: Asset = asset!("/assets/library.json");

fn main() {
    dioxus::launch(App);
}

/// First visit: a demo current song and queue so the booth is never empty
fn demo_session() -> Session {
    let cat = builtin_catalog();
    let item = |queue_id: u64, index: usize, requester: &str| QueueItem {
        queue_id,
        song: cat[index].clone(),
        requester: requester.to_string(),
    };
    Session {
        current: Some(item(1, 3, "KTV Host")), // โจอี้ ภูวศิษฐ์ - รักไม่ไหวแล้วโว้ย
        queue: vec![
            item(2, 2, "Table 1"), // เสือ ธนพล - นกหลงรัง
            item(3, 6, "Table 1"), // LULA - ดาวเสาร์
            item(4, 12, "VIP"),    // Atom - oasis
        ],
        next_queue_id: 5,
        custom_songs: Vec::new(),
    }
}

#[component]
fn App() -> Element {
    let mut settings = use_signal(|| storage::load::<AppSettings>(SETTINGS_KEY).unwrap_or_default());
    // Open on every visit until closed once
    let mut show_help = use_signal(move || !settings.peek().seen_shortcuts);
    let mut active_tab = use_signal(|| KtvTab::Catalog);

    // Guide timings set in timing mode on this device; they win over the catalog's
    let mut guide_overrides = use_signal(|| storage::load::<GuideOverrides>(GUIDES_KEY).unwrap_or_default());

    // Resume the last session (reconciled with this build's catalog), or start with the demo queue
    let restored = use_hook(|| {
        let mut session =
            storage::load::<Session>(SESSION_KEY).map_or_else(demo_session, |s| s.reconcile(builtin_catalog()));
        let songs = session.current.iter_mut().chain(&mut session.queue).map(|it| &mut it.song);
        timing::apply_overrides(songs, &guide_overrides.peek());
        Rc::new(session)
    });
    let mut catalog = use_signal({
        let restored = restored.clone();
        move || {
            let mut songs: Vec<Song> = builtin_catalog().iter().chain(&restored.custom_songs).cloned().collect();
            timing::apply_overrides(songs.iter_mut(), &guide_overrides.peek());
            songs
        }
    });
    let mut booth = use_signal(move || Booth {
        current: restored.current.clone(),
        queue: restored.queue.clone(),
        next_queue_id: restored.next_queue_id,
    });
    // Slices of the booth: readers re-render only when their part changes
    let current_song = use_memo(move || booth.read().current.clone());
    let queue = use_memo(move || booth.read().queue.clone());

    // Persist on change (effects re-run when the signals they read are written)
    use_effect(move || storage::save(SETTINGS_KEY, &*settings.read()));
    use_effect(move || storage::save(GUIDES_KEY, &*guide_overrides.read()));
    // Finished takes (newest first) and the one just finished, shown until closed
    let mut score_history = use_signal(|| storage::load::<Vec<TakeResult>>(SCORES_KEY).unwrap_or_default());
    let mut last_result = use_signal(|| None::<TakeResult>);
    use_effect(move || storage::save(SCORES_KEY, &*score_history.read()));
    // Favourites and recently sung songs on this device
    let mut picks = use_signal(|| storage::load::<Picks>(PICKS_KEY).unwrap_or_default());
    use_effect(move || storage::save(PICKS_KEY, &*picks.read()));
    use_effect(move || {
        let b = booth.read();
        let session = Session::capture(b.current.clone(), b.queue.clone(), b.next_queue_id, &catalog.read(), builtin_catalog());
        storage::save(SESSION_KEY, &session);
    });

    let mut playback_speed = use_signal(|| 1.0f32);
    // Player's per-song "Play Intro" choice; seeded from settings on each new song, read by replay
    let intro_skipped = use_signal(|| true);
    let mut anticipator = use_signal(SleepTimeAnticipator::new);
    let mut song_started_at = use_signal(js_sys::Date::now);
    // One-line toast: Auto-DJ picks, songs queued by keypad code
    let mut booth_notice = use_signal(|| None::<String>);
    let mut search_query = use_signal(String::new);

    // Full library: the curated catalog is usable at once; the rest joins when the fetch lands
    let mut song_library = use_signal(library::loaded);
    use_future(move || async move {
        let Some(json) = app::browser::fetch_text(&LIBRARY_JSON.to_string()).await else { return };
        match library::install(&json) {
            Ok(loaded) => song_library.set(loaded),
            Err(err) => dioxus::logger::tracing::warn!("assets/library.json unreadable: {err}"),
        }
    });

    // Fullscreen player: the songbook is out of view, so typed searches list beside the video
    let mut is_fullscreen = use_signal(|| false);
    use_hook(move || {
        let mut changes = app::browser::watch_fullscreen();
        spawn(async move {
            while let Some(on) = changes.next().await {
                is_fullscreen.set(on);
            }
        });
    });

    // Sleep-time compute (pre-anticipates next recommended songs during playback)
    // Candidates: the booth's catalog first, then the full library once it has loaded
    let anticipated_set = use_memo(move || {
        let queued_ids: HashSet<String> = queue.read().iter().map(|it| it.song.id.clone()).collect();
        let curr_id = current_song().map(|c| c.song.id);
        let songs = catalog.read();
        let candidates = songs.iter().chain(song_library().songs());
        anticipator.read().sleep_compute(candidates, &queued_ids, curr_id.as_deref(), 4)
    });

    // Helper: Finish song and transition to next or Auto-DJ
    let mut transition_to_next = move |completed_natural: bool| {
        let elapsed_secs = ((js_sys::Date::now() - song_started_at()) / 1000.0).max(1.0);

        // Record telemetry for the finished/skipped song
        if let Some(curr) = current_song() {
            picks.write().record_sung(&curr.song.id, elapsed_secs);
            let tele = SongTelemetry {
                song_id: curr.song.id.clone(),
                code: curr.song.code.clone(),
                title: curr.song.title.clone(),
                artist: curr.song.artist.clone(),
                category: curr.song.category.clone(),
                duration_secs: curr.song.duration_secs,
                sang_seconds: elapsed_secs,
                completed_natural,
            };
            let mut ant = anticipator();
            ant.record_song_playback(tele);
            anticipator.set(ant);
        }

        // Reset timer
        song_started_at.set(js_sys::Date::now());

        // Picks computed while the finished song is still on stage, so Auto-DJ never repeats it
        let ant_set = anticipated_set();
        if booth.write().advance() {
            booth_notice.set(None);
        } else if let Some(rec) = anticipator().wake_consume(&ant_set) {
            // Queue is empty: Auto-DJ plays the top anticipated pick
            let (title, artist, reason) = (&rec.song.title, &rec.song.artist, &rec.reason);
            booth_notice.set(Some(format!("🧠 Auto-DJ: queue is empty, playing next: {title} - {artist} ({reason})")));
            let mut song = rec.song;
            timing::apply_overrides([&mut song], &guide_overrides.peek());
            booth.write().add(song, Requester::AutoDj, Placement::Now);
        }
    };

    // Next / Skip Song Handler (manual skip)
    let handle_next_song = move |_: ()| {
        transition_to_next(false);
    };

    // Video natural ended handler (via YouTube onStateChange: 0)
    let handle_video_ended = move |_: ()| {
        transition_to_next(true);
    };

    // Replay current song in place (same iframe): seek to its start and resume
    let handle_replay_song = move |_: ()| {
        if let Some(curr) = current_song() {
            song_started_at.set(js_sys::Date::now());
            SyncCommand::Restart(curr.song.start_sec(intro_skipped())).run();
        }
    };

    // Every way of requesting a song goes through here; a song that goes on stage starts a new take
    let mut request = move |mut song: Song, requester: Requester, placement: Placement| {
        // Library songs are not in `catalog`, so a guide timed on this device is applied as they are requested
        timing::apply_overrides([&mut song], &guide_overrides.peek());
        if booth.write().add(song, requester, placement) {
            song_started_at.set(js_sys::Date::now());
        }
    };
    let song_by_code =
        move |code: &str| catalog::find_song(&catalog.read(), song_library(), |s| s.code == code).cloned();

    let handle_play_song = move |song: Song| request(song, Requester::Singer, Placement::Now);
    let handle_queue_song = move |song: Song| request(song, Requester::Guest, Placement::Back);
    let handle_queue_next_song = move |song: Song| request(song, Requester::Priority, Placement::Next);
    let handle_play_by_code = move |code: String| {
        if let Some(song) = song_by_code(&code) {
            request(song, Requester::Keypad, Placement::Now);
        }
    };
    let handle_queue_by_code = move |code: String| {
        if let Some(song) = song_by_code(&code) {
            request(song, Requester::Keypad, Placement::Back);
        }
    };

    // Booth keyboard and gamepad: type-to-search, Enter queues a typed code, Space pause, arrows seek, ? help,
    // next song (assets/ktv_keys.js)
    use_effect(move || {
        let mut messages = keys::install();
        spawn(async move {
            while let Some(msg) = messages.next().await {
                let Some(action) = KeyAction::parse(&msg) else { continue };
                match action {
                    KeyAction::Type(c) => {
                        search_query.write().push(c);
                        active_tab.set(KtvTab::Catalog);
                    }
                    KeyAction::Backspace => {
                        search_query.write().pop();
                        active_tab.set(KtvTab::Catalog);
                    }
                    KeyAction::ClearSearch => search_query.set(String::new()),
                    KeyAction::TogglePlayback => SyncCommand::TogglePlayback.run(),
                    KeyAction::SeekBy(secs) => SyncCommand::SeekBy(secs).run(),
                    KeyAction::ToggleHelp => show_help.toggle(),
                    KeyAction::Submit => {
                        let query = search_query();
                        let Some(code) = catalog::keypad_code(&query) else { continue };
                        match song_by_code(code) {
                            Some(song) => {
                                booth_notice.set(Some(format!("Queued {code}: {} - {}", song.title, song.artist)));
                                request(song, Requester::Keypad, Placement::Back);
                                search_query.set(String::new());
                            }
                            None => booth_notice.set(Some(format!("No song with code {code}"))),
                        }
                    }
                    KeyAction::NextSong => transition_to_next(false),
                }
            }
        });
    });

    let handle_move_up = move |index: usize| booth.write().move_up(index);
    let handle_move_down = move |index: usize| booth.write().move_down(index);
    let handle_remove_queue = move |queue_id: u64| booth.write().remove(queue_id);
    let handle_clear_queue = move |_: ()| booth.write().clear_queue();

    // Add custom song from YouTube
    // Returns the stored song (with its keypad code) so the form can report success or failure
    let handle_add_custom_song = move |(new_song, play_now): (Song, bool)| -> Result<Song, catalog::CustomCodesExhausted> {
        // A curated or library video keeps its own entry (code, guide, intro skip); anything else
        // gets a unique keypad code, and re-adding the same video reuses its entry
        let known = catalog::songbook_video(&catalog.read(), song_library(), &new_song.youtube_id).cloned();
        let new_song = match known {
            Some(song) => song,
            None => catalog::upsert_custom(&mut catalog.write(), new_song)?,
        };
        let placement = if play_now { Placement::Now } else { Placement::Back };
        request(new_song.clone(), Requester::AddUrl, placement);
        Ok(new_song)
    };

    // Put a guide on every copy of a song: catalog, current song, queue
    let mut set_song_guide = move |song_id: &str, guide: Option<GuideTrack>| {
        for song in catalog.write().iter_mut().filter(|s| s.id == song_id) {
            song.guide = guide.clone();
        }
        booth.write().set_guide(song_id, guide.as_ref());
    };

    let handle_save_guide = move |guide: GuideTrack| {
        let Some(song_id) = current_song().map(|c| c.song.id) else { return };
        guide_overrides.write().insert(song_id.clone(), guide.clone());
        set_song_guide(&song_id, Some(guide));
    };

    // The curated catalog's timing, or a library song's timed official video (assets/mv_guides.json)
    let catalog_guide = |song_id: &str| match builtin_catalog().iter().find(|s| s.id == song_id) {
        Some(song) => song.guide.clone(),
        None => song_id.strip_prefix(library::ID_PREFIX).and_then(library::timed_guide),
    };

    // Back to the catalog timing (custom songs have none, so their guide is dropped)
    let handle_revert_guide = move |_: ()| {
        let Some(song_id) = current_song().map(|c| c.song.id) else { return };
        guide_overrides.write().remove(&song_id);
        set_song_guide(&song_id, catalog_guide(&song_id));
    };

    let saved_timing = match current_song().map(|c| c.song.id) {
        Some(id) if guide_overrides.read().contains_key(&id) => match catalog_guide(&id) {
            Some(_) => SavedTiming::OverCatalog,
            None => SavedTiming::Only,
        },
        _ => SavedTiming::None,
    };
    let ant_candidates = anticipated_set().candidates;

    rsx! {

        div { class: if settings().tv_mode { "ktv-app-wrapper tv-mode" } else { "ktv-app-wrapper" },
            // Song changes announced to screen readers
            div { class: "sr-only", role: "status", aria_live: "polite",
                if let Some(curr) = current_song() {
                    span { lang: "th", "Now singing: {curr.song.title} by {curr.song.artist}" }
                }
            }
            // Header Bar
            Header {
                active_tab,
                queue_len: queue().len(),
                room_name: settings().room_name.clone(),
                on_help: move |_| show_help.toggle(),
                tv_mode: settings().tv_mode,
                on_toggle_tv: move |_| {
                    let on = !settings.peek().tv_mode;
                    settings.write().tv_mode = on;
                },
            }

            // Auto-DJ Toast Notification
            if let Some(notice) = booth_notice() {
                div { class: "auto-dj-toast",
                    span { class: "toast-icon", "✨" }
                    span { "{notice}" }
                    button {
                        class: "toast-close-btn",
                        onclick: move |_| booth_notice.set(None),
                        "✕"
                    }
                }
            }

            // Main Split Stage
            main { class: "ktv-split-stage",
                // Left / Main Player Viewport
                section { class: "stage-player-side",
                    Player {
                        current_item: current_song(),
                        take_started_at: song_started_at(),
                        auto_skip_intro: settings().auto_skip_intro,
                        is_skipped: intro_skipped,
                        playback_speed: playback_speed(),
                        on_next_song: handle_next_song,
                        on_replay_song: handle_replay_song,
                        on_video_ended: handle_video_ended,
                        show_timing_tools: settings().show_timing_tools,
                        saved_timing,
                        on_save_guide: handle_save_guide,
                        on_revert_guide: handle_revert_guide,
                        auto_timed: current_song().is_some_and(|c| {
                            !guide_overrides.read().contains_key(&c.song.id) && library::auto_timed(&c.song.youtube_id)
                        }),
                        on_take_end: move |result: TakeResult| {
                            score::record(&mut score_history.write(), result.clone());
                            last_result.set(Some(result));
                        },
                    }
                    if settings().tv_mode {
                        UpNext { queue: queue() }
                    }
                    if is_fullscreen() && !search_query.read().is_empty() {
                        QuickSearch {
                            catalog: catalog(),
                            library: song_library(),
                            search_query,
                            on_play_song: handle_play_song,
                            on_queue_song: handle_queue_song,
                            on_queue_next_song: handle_queue_next_song,
                        }
                    }
                }

                // Right / Tabbed Controller Panel
                section { class: "stage-control-side",
                    if let Some(result) = last_result() {
                        ScoreCard { result, on_close: move |_| last_result.set(None) }
                    }
                    if show_help() {
                        ShortcutHelp {
                            on_close: move |_| {
                                show_help.set(false);
                                if !settings.peek().seen_shortcuts {
                                    settings.write().seen_shortcuts = true;
                                }
                            },
                        }
                    }
                    match active_tab() {
                        KtvTab::Catalog => rsx! {
                            CatalogView {
                                catalog: catalog(),
                                library: song_library(),
                                search_query,
                                on_play_song: handle_play_song,
                                on_queue_song: handle_queue_song,
                                on_queue_next_song: handle_queue_next_song,
                                picks: picks(),
                                on_toggle_favourite: move |id: String| picks.write().toggle_favourite(&id),
                            }
                        },
                        KtvTab::Queue => rsx! {
                            QueueView {
                                queue: queue(),
                                current_item: current_song(),
                                anticipated: ant_candidates,
                                score_history: score_history(),
                                on_skip: handle_next_song,
                                on_remove: handle_remove_queue,
                                on_move_up: handle_move_up,
                                on_move_down: handle_move_down,
                                on_clear_queue: handle_clear_queue,
                                on_queue_song: handle_queue_song,
                                on_play_song: handle_play_song,
                                on_simulate_end: handle_video_ended,
                                show_dev_tools: settings().show_timing_tools,
                            }
                        },
                        KtvTab::Remote => rsx! {
                            Remote {
                                catalog: catalog(),
                                library: song_library(),
                                on_play_by_code: handle_play_by_code,
                                on_queue_by_code: handle_queue_by_code,
                                on_skip_song: handle_next_song,
                                on_replay_song: handle_replay_song,
                                on_speed_change: move |s| playback_speed.set(s),
                                current_speed: playback_speed(),
                            }
                        },
                        KtvTab::CustomAdd => rsx! {
                            CustomAdd {
                                on_add_song: handle_add_custom_song,
                            }
                        },
                        KtvTab::Settings => rsx! {
                            Settings {
                                settings,
                            }
                        },
                    }
                }
            }
        }
    }
}
