use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::HashSet;
use std::rc::Rc;

use app::booth::{self, Booth, Placement, Requester};
use app::catalog;
use app::components;
use app::keys::{self, KeyAction};
use app::library;
use app::mc::{self, McEvent};
use app::medley::{self, Medley, MedleyBook, SharedMedley};
use app::recommendation;
use app::room::{self, PhoneCommand};
use app::picks::Picks;
use app::score::{self, TakeResult};
use app::storage::{self, Session, GUIDES_KEY, MEDLEYS_KEY, PICKS_KEY, SCORES_KEY, SESSION_KEY, SETTINGS_KEY};
use app::sync::SyncCommand;
use app::timing::{self, GuideOverrides, SavedTiming};
use app::tip::{self, ConfirmedTip, TipAction};
use app::types;

use catalog::builtin_catalog;
use components::{
    catalog_view::CatalogView,
    custom_add::CustomAdd,
    header::Header,
    medley::MedleyPanel,
    phone_remote::{use_room_link, PhoneRemotePanel},
    player::Player,
    queue_view::QueueView,
    quick_search::QuickSearch,
    remote::Remote,
    settings::Settings,
    score_card::ScoreCard,
    shortcuts::ShortcutHelp,
    tip_qr::{self, TipQr, TipRequestWatch},
    tip_toast::TipToast,
    up_next::UpNext,
};
use recommendation::{SleepTimeAnticipator, SongTelemetry};
use types::{AppSettings, GuideTrack, KtvTab, QueueItem, Song};

// In the static <head> at build time: the stylesheet loads alongside the wasm instead of after it has run
const _: Asset = asset!("/assets/main.css", AssetOptions::css().with_static_head(true));
// Classic scripts in the static <head>: they run before the wasm, so Rust calls them directly (no eval, see js_bridge)
const _: Asset = asset!("/assets/ktv_sync.js", AssetOptions::js().with_static_head(true));
const _: Asset = asset!("/assets/ktv_keys.js", AssetOptions::js().with_static_head(true));
// The full songbook and its songs' original-vocal guides, fetched after the first paint (content-hashed, so cached for good)
const LIBRARY_JSON: Asset = asset!("/assets/library.json");
const MV_GUIDES_JSON: Asset = asset!("/assets/mv_guides.json");
/// Volume change per tap on a phone remote (a phone is tapped less often than a key is pressed).
const PHONE_VOLUME_STEP: i32 = 10;
/// The MC waits this long before announcing a song, so the last take's score is said first.
const MC_SONG_UP_DELAY_MS: i32 = 400;

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
        part: None,
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
    // A medley shared by link (`#medley=…`): read once, then dropped from the address bar so a reload does not
    // offer it again; the page opens on the Queue tab, where the medley builder offers it
    let shared_medley = use_signal(|| {
        let shared = app::browser::page_fragment().and_then(|f| medley::parse_fragment(&f));
        if shared.is_some() {
            app::browser::clear_fragment();
        }
        shared
    });
    let mut active_tab =
        use_signal(move || if shared_medley.peek().is_some() { KtvTab::Queue } else { KtvTab::Catalog });

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
    // Result cards of the take that just ended: one, or one per singer in a duet
    let mut last_results = use_signal(Vec::<TakeResult>::new);
    use_effect(move || storage::save(SCORES_KEY, &*score_history.read()));
    // Favourites and recently sung songs on this device
    let mut picks = use_signal(|| storage::load::<Picks>(PICKS_KEY).unwrap_or_default());
    use_effect(move || storage::save(PICKS_KEY, &*picks.read()));
    // Medleys (plan 004): shared through context so the practice row can add a marked part
    let mut medleys = use_signal(|| storage::load::<MedleyBook>(MEDLEYS_KEY).unwrap_or_default().sanitized());
    use_effect(move || storage::save(MEDLEYS_KEY, &*medleys.read()));
    use_context_provider(|| medleys);
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
    use_hook(move || {
        if let Some(shared) = &*shared_medley.peek() {
            let title = medley::shown_title(&shared.title);
            booth_notice.set(Some(format!("🔗 Shared medley: {title}. Open it under MEDLEY below the queue")));
        }
    });
    let mut search_query = use_signal(String::new);

    // Full library: the curated catalog is usable at once; the rest joins when the fetch lands
    let mut song_library = use_signal(library::loaded);
    use_future(move || async move {
        let (guides, json) = futures_util::future::join(
            app::browser::fetch_text(&MV_GUIDES_JSON.to_string()),
            app::browser::fetch_text(&LIBRARY_JSON.to_string()),
        )
        .await;
        // Guides first: library songs take their Vocal timing from them. Without them the library still works.
        match guides.map(|guides| library::install_guides(&guides)) {
            Some(Ok(())) => {}
            Some(Err(err)) => dioxus::logger::tracing::warn!("assets/mv_guides.json unreadable: {err}"),
            None => dioxus::logger::tracing::warn!("assets/mv_guides.json not fetched"),
        }
        let Some(json) = json else { return };
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
    let mut handle_replay_song = move |_: ()| {
        if let Some(curr) = current_song() {
            song_started_at.set(js_sys::Date::now());
            SyncCommand::Restart(curr.start_at(intro_skipped())).run();
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

    // A medley goes in as one entry per part, back to back; every part's song must be found first
    let handle_queue_medley = move |(medley, placement): (Medley, Placement)| -> Result<String, String> {
        let slots = medley::slots(&medley).map_err(|e| e.message())?;
        let mut parts = Vec::with_capacity(slots.len());
        for (song_id, slot) in slots {
            let found = catalog::find_song(&catalog.read(), song_library(), |s| s.id == song_id).cloned();
            let Some(mut song) = found else {
                return Err("A song in this medley is not in the songbook (yet): wait for the library to load, or remove it".to_string());
            };
            timing::apply_overrides([&mut song], &guide_overrides.peek());
            parts.push((song, slot));
        }
        let (title, count) = (medley::display_title(&medley).to_string(), parts.len());
        if booth.write().add_medley(parts, Requester::Singer, placement) {
            song_started_at.set(js_sys::Date::now());
            return Ok(format!("Playing {title} ({count} parts)"));
        }
        Ok(format!("Queued {title} ({count} parts)"))
    };
    let handle_open_shared_medley = move |shared: SharedMedley| shared.open(song_by_code);
    let handle_add_medley_code = move |code: String| -> Result<String, String> {
        let code = catalog::keypad_code(&code).ok_or("Type a 5-digit song code")?;
        let song = song_by_code(code).ok_or(format!("No song with code {code}"))?;
        medleys.write().add_song(&song).map_err(|e| e.message())?;
        Ok(format!("Added {} - {}", song.title, song.artist))
    };

    let handle_play_song = move |song: Song| request(song, Requester::Singer, Placement::Now);
    let handle_queue_song = move |song: Song| request(song, Requester::Guest, Placement::Back);
    let handle_queue_next_song = move |song: Song| request(song, Requester::Priority, Placement::Next);
    // Plan 003 A3: the MC announces each song that goes on stage (not the one restored on load, not a replay).
    // A short wait lets the finished take's score be said first.
    let mut announced = use_signal(|| booth.peek().current.as_ref().map(|c| c.queue_id));
    use_hook(|| {
        if settings.peek().mc_voice != app::mc::McVoice::Off {
            app::browser::load_voices();
        }
    });
    use_effect(move || {
        let Some(item) = current_song() else { return };
        if *announced.peek() == Some(item.queue_id) {
            return;
        }
        announced.set(Some(item.queue_id));
        // A medley's later parts follow straight on: the MC speaks before its first part only
        if item.is_medley_join() {
            return;
        }
        let voice = settings.peek().mc_voice;
        let tipped = booth::is_tip_request(&item);
        let event = McEvent::SongUp { title: item.song.title.clone(), artist: item.song.artist.clone(), tipped };
        let seed = mc::seed_of(&item.song.id);
        spawn(async move {
            app::browser::sleep_ms(MC_SONG_UP_DELAY_MS).await;
            mc::announce(&event, voice, seed);
        });
    });

    // Plan 003 S3: tips from both QRs. A request memo with enough USDC queues its song next as ★ TIP; every tip
    // gets a garland toast. Each signature is acted on once, even when a watcher restarts and sees it again.
    let room_reference =
        use_hook(|| tip::reference_bytes().map(|bytes| tip::reference_from_bytes(&bytes)).unwrap_or_default());
    let mut tips_handled = use_signal(HashSet::<String>::new);
    let mut tip_notice = use_signal(|| None::<(String, String)>);
    let handle_tip = move |confirmed: ConfirmedTip| {
        if !tips_handled.write().insert(confirmed.signature.clone()) {
            return;
        }
        let amount = tip::format_usdc(confirmed.amount);
        let text = match tip::tip_action(&confirmed) {
            TipAction::Request(code) => match song_by_code(&code) {
                Some(song) => {
                    let text = format!("★ TIP {amount} USDC: {} - {} plays next", song.title, song.artist);
                    request(song, Requester::Tip, Placement::Next);
                    text
                }
                None => format!("Garland for the singer! +{amount} USDC (no song with code {code})"),
            },
            TipAction::Garland => format!("Garland for the singer! +{amount} USDC"),
        };
        tip_notice.set(Some((confirmed.signature, text)));
    };
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

    // Booth volume: the sync core applies it to both players and to every player that loads later
    let volume = use_memo(move || settings().volume());
    use_effect(move || SyncCommand::SetVolume(volume()).run());

    // Phone remote: guests queue songs from `/remote`; skip / pause / replay only if the host allows it
    let remote_config = use_memo(move || settings().remote);
    let room_state = use_memo(move || {
        let playback = remote_config().allow_playback;
        room::booth_state(&settings().room_name, current_song().as_ref(), &queue(), playback, volume())
    });
    let on_phone_command = use_callback(move |cmd: PhoneCommand| -> Result<String, String> {
        if let Some(reason) = room::refusal(&cmd, &remote_config.peek()) {
            return Err(reason.to_string());
        }
        match cmd {
            PhoneCommand::Queue { code } => {
                let song = catalog::keypad_code(&code).and_then(song_by_code).ok_or(format!("No song with code {code}"))?;
                let text = format!("Queued {code}: {} - {}", song.title, song.artist);
                request(song, Requester::Phone, Placement::Back);
                booth_notice.set(Some(format!("📱 {text}")));
                Ok(text)
            }
            PhoneCommand::Skip => {
                transition_to_next(false);
                Ok("Skipped to the next song".to_string())
            }
            PhoneCommand::Pause => {
                SyncCommand::TogglePlayback.run();
                Ok("Play / pause".to_string())
            }
            PhoneCommand::Replay => {
                handle_replay_song(());
                Ok("Replaying from the start".to_string())
            }
            PhoneCommand::VolumeUp => Ok(format!("Volume {}", settings.write().volume_by(PHONE_VOLUME_STEP))),
            PhoneCommand::VolumeDown => Ok(format!("Volume {}", settings.write().volume_by(-PHONE_VOLUME_STEP))),
        }
    });
    let room_link = use_room_link(remote_config, room_state, on_phone_command);

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
                    KeyAction::VolumeBy(delta) => {
                        settings.write().volume_by(delta);
                    }
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

            if let Some((signature, text)) = tip_notice() {
                TipToast { key: "{signature}", text, on_close: move |_| tip_notice.set(None) }
            }
            if !room_reference.is_empty() {
                TipRequestWatch { config: settings().tip, reference: room_reference.clone(), on_tip: handle_tip }
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
                        volume: settings().volume(),
                        on_volume: move |v: u32| settings.write().volume = v,
                        duet: settings().duet,
                        mc_on: settings().mc_voice != app::mc::McVoice::Off,
                        auto_timed: current_song().is_some_and(|c| {
                            !guide_overrides.read().contains_key(&c.song.id) && library::auto_timed(&c.song.youtube_id)
                        }),
                        on_take_end: move |result: TakeResult| {
                            // Between medley parts the MC stays quiet (the next part is already playing)
                            let joined = current_song().is_some_and(|c| c.is_medley_join());
                            if let (Some(score), false) = (result.score(), joined) {
                                let event = McEvent::TakeEnded { score, singer: result.singer.clone() };
                                mc::announce(&event, settings.peek().mc_voice, mc::seed_of(&result.song_id));
                            }
                            score::record(&mut score_history.write(), result.clone());
                            // Parts of one duet take end together; a new take replaces the cards
                            let mut cards = last_results.write();
                            if cards.first().is_some_and(|c| c.sung_at_ms != result.sung_at_ms) {
                                cards.clear();
                            }
                            cards.push(result);
                        },
                    }
                    if let Some(item) = current_song() {
                        TipQr {
                            item,
                            config: settings().tip,
                            room_name: settings().room_name,
                            request_page: tip_qr::request_page(&settings().tip, &room_reference, &settings().room_name),
                            on_tip: handle_tip,
                        }
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
                    for (i, result) in last_results().into_iter().enumerate() {
                        ScoreCard {
                            key: "{result.sung_at_ms}-{result.part.unwrap_or(0)}",
                            result,
                            singers: score::recent_singers(&score_history.read(), 6),
                            on_name: move |name: Option<String>| {
                                let mut cards = last_results.write();
                                let Some(result) = cards.get_mut(i) else { return };
                                result.singer = name.as_deref().and_then(score::clean_name);
                                score::name_take(&mut score_history.write(), result.sung_at_ms, result.part, result.singer.clone());
                            },
                            on_close: move |_| {
                                last_results.write().remove(i);
                            },
                        }
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
                                leaders: score::leaderboard(&score_history.read(), js_sys::Date::now()),
                                on_skip: handle_next_song,
                                on_remove: handle_remove_queue,
                                on_move_up: handle_move_up,
                                on_move_down: handle_move_down,
                                on_clear_queue: handle_clear_queue,
                                on_queue_song: handle_queue_song,
                                on_play_song: handle_play_song,
                                on_simulate_end: handle_video_ended,
                                show_dev_tools: settings().show_timing_tools,
                                MedleyPanel {
                                    on_add_code: handle_add_medley_code,
                                    on_queue: handle_queue_medley,
                                    shared: shared_medley,
                                    on_open_shared: handle_open_shared_medley,
                                }
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
                                PhoneRemotePanel {
                                config: remote_config(),
                                status: (room_link.status)(),
                                phone_url: (room_link.phone_url)(),
                                on_config: move |config| settings.write().remote = config,
                                    on_new_link: move |_| room_link.new_link(),
                                }
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
