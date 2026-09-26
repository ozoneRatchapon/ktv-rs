use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

use app::booth::{Booth, Placement, Requester};
use app::catalog;
use app::components;
use app::keys::{self, KeyAction};
use app::recommendation;
use app::storage::{self, Session, GUIDES_KEY, SESSION_KEY, SETTINGS_KEY};
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
    remote::Remote,
    settings::Settings,
    shortcuts::ShortcutHelp,
};
use recommendation::{SleepTimeAnticipator, SongTelemetry};
use types::{AppSettings, GuideTrack, KtvTab, QueueItem, Song};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const FAVICON: Asset = asset!("/assets/favicon.ico");

fn main() {
    dioxus::launch(App);
}

/// First visit: a demo current song and queue so the booth is never empty
fn demo_session() -> Session {
    let cat = builtin_catalog();
    let item = |queue_id: u64, index: usize, requester: &str| QueueItem {
        queue_id,
        song: cat[index].clone(),
        key_shift: 0,
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
    let mut auto_dj_notice = use_signal(|| None::<String>);
    let mut search_query = use_signal(String::new);

    // Booth keyboard: type-to-search, Space pause, arrows seek, ? help (assets/ktv_keys.js)
    use_effect(move || {
        let mut eval = keys::install();
        spawn(async move {
            while let Ok(msg) = eval.recv::<String>().await {
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
                }
            }
        });
    });

    // Sleep-time compute (pre-anticipates next recommended songs during playback)
    let anticipated_set = use_memo(move || {
        let cat = catalog();
        let q = queue();
        let queued_ids: HashSet<String> = q.iter().map(|it| it.song.id.clone()).collect();
        let curr_id = current_song().map(|c| c.song.id);

        let ant = anticipator();
        ant.sleep_compute(&cat, &queued_ids, curr_id.as_deref(), 4)
    });

    // Helper: Finish song and transition to next or Auto-DJ
    let mut transition_to_next = move |completed_natural: bool| {
        let elapsed_secs = ((js_sys::Date::now() - song_started_at()) / 1000.0).max(1.0);

        // Record telemetry for the finished/skipped song
        if let Some(curr) = current_song() {
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
            auto_dj_notice.set(None);
        } else if let Some(rec) = anticipator().wake_consume(&ant_set) {
            // Queue is empty: Auto-DJ plays the top anticipated pick
            let (title, artist, reason) = (&rec.song.title, &rec.song.artist, &rec.reason);
            auto_dj_notice.set(Some(format!("🧠 Auto-DJ: queue is empty, playing next: {title} - {artist} ({reason})")));
            booth.write().add(rec.song, Requester::AutoDj, Placement::Now);
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
    let mut request = move |song: Song, requester: Requester, placement: Placement| {
        if booth.write().add(song, requester, placement) {
            song_started_at.set(js_sys::Date::now());
        }
    };
    let song_by_code = move |code: &str| catalog.read().iter().find(|s| s.code == code).cloned();

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

    let handle_key_change = move |delta: i32| booth.write().shift_current_key(delta);
    let handle_reset_key = move |_: ()| booth.write().reset_current_key();
    let handle_adjust_item_key = move |(queue_id, delta): (u64, i32)| booth.write().shift_item_key(queue_id, delta);
    let handle_move_up = move |index: usize| booth.write().move_up(index);
    let handle_move_down = move |index: usize| booth.write().move_down(index);
    let handle_remove_queue = move |queue_id: u64| booth.write().remove(queue_id);
    let handle_clear_queue = move |_: ()| booth.write().clear_queue();

    // Add custom song from YouTube
    // Returns the stored song (with its keypad code) so the form can report success or failure
    let handle_add_custom_song = move |(new_song, play_now): (Song, bool)| -> Result<Song, catalog::CustomCodesExhausted> {
        // Assigns a unique keypad code; re-adding the same video reuses its entry
        let new_song = catalog::upsert_custom(&mut catalog.write(), new_song)?;
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

    let catalog_guide = |song_id: &str| builtin_catalog().iter().find(|s| s.id == song_id).and_then(|s| s.guide.clone());

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
    let current_key = current_song().map(|c| c.key_shift).unwrap_or(0);
    let ant_candidates = anticipated_set().candidates;

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div { class: "ktv-app-wrapper",
            // Header Bar
            Header {
                active_tab,
                queue_len: queue().len(),
                room_name: settings().room_name.clone(),
                on_help: move |_| show_help.toggle(),
            }

            // Auto-DJ Toast Notification
            if let Some(notice) = auto_dj_notice() {
                div { class: "auto-dj-toast",
                    span { class: "toast-icon", "✨" }
                    span { "{notice}" }
                    button {
                        class: "toast-close-btn",
                        onclick: move |_| auto_dj_notice.set(None),
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
                        on_key_change: handle_key_change,
                        on_video_ended: handle_video_ended,
                        show_timing_tools: settings().show_timing_tools,
                        saved_timing,
                        on_save_guide: handle_save_guide,
                        on_revert_guide: handle_revert_guide,
                    }
                }

                // Right / Tabbed Controller Panel
                section { class: "stage-control-side",
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
                                search_query,
                                on_play_song: handle_play_song,
                                on_queue_song: handle_queue_song,
                                on_queue_next_song: handle_queue_next_song,
                            }
                        },
                        KtvTab::Queue => rsx! {
                            QueueView {
                                queue: queue(),
                                current_item: current_song(),
                                anticipated: ant_candidates,
                                on_skip: handle_next_song,
                                on_remove: handle_remove_queue,
                                on_move_up: handle_move_up,
                                on_move_down: handle_move_down,
                                on_clear_queue: handle_clear_queue,
                                on_adjust_item_key: handle_adjust_item_key,
                                on_queue_song: handle_queue_song,
                                on_play_song: handle_play_song,
                                on_simulate_end: handle_video_ended,
                            }
                        },
                        KtvTab::Remote => rsx! {
                            Remote {
                                catalog: catalog(),
                                on_play_by_code: handle_play_by_code,
                                on_queue_by_code: handle_queue_by_code,
                                on_skip_song: handle_next_song,
                                on_replay_song: handle_replay_song,
                                on_key_shift: handle_key_change,
                                on_reset_key: handle_reset_key,
                                on_speed_change: move |s| playback_speed.set(s),
                                current_key,
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
