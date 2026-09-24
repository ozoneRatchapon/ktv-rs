use dioxus::prelude::*;
use std::collections::HashSet;

use app::catalog;
use app::components;
use app::recommendation;
use app::types;

use catalog::get_initial_catalog;
use components::{
    catalog_view::CatalogView,
    custom_add::CustomAdd,
    header::Header,
    player::Player,
    queue_view::QueueView,
    remote::Remote,
    settings::Settings,
};
use recommendation::{SleepTimeAnticipator, SongTelemetry};
use types::{AppSettings, KtvTab, QueueItem, Song};

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let settings = use_signal(AppSettings::default);
    let mut active_tab = use_signal(|| KtvTab::Catalog);
    let mut catalog = use_signal(get_initial_catalog);

    // Initial default song (Joey Phuwasit - รักไม่ไหวแล้วโว้ย)
    let initial_song = catalog().get(3).cloned().unwrap_or_else(|| catalog()[0].clone());
    let mut current_song = use_signal(move || {
        Some(QueueItem {
            queue_id: 1,
            song: initial_song,
            key_shift: 0,
            requester: "KTV Host".to_string(),
        })
    });

    let mut queue = use_signal(|| {
        vec![
            QueueItem {
                queue_id: 2,
                song: get_initial_catalog()[2].clone(), // เสือ ธนพล - นกหลงรัง
                key_shift: 0,
                requester: "Table 1".to_string(),
            },
            QueueItem {
                queue_id: 3,
                song: get_initial_catalog()[6].clone(), // LULA - ดาวเสาร์
                key_shift: 0,
                requester: "Table 1".to_string(),
            },
            QueueItem {
                queue_id: 4,
                song: get_initial_catalog()[12].clone(), // Atom - oasis
                key_shift: 0,
                requester: "VIP".to_string(),
            },
        ]
    });

    let mut next_queue_id = use_signal(|| 5u64);
    let mut playback_speed = use_signal(|| 1.0f32);
    let mut anticipator = use_signal(SleepTimeAnticipator::new);
    let mut song_started_at = use_signal(js_sys::Date::now);
    let mut auto_dj_notice = use_signal(|| None::<String>);
    let mut search_query = use_signal(String::new);

    // Global Keypress Listener (Type-to-Search & Space to Pause like authentic KTV booth)
    use_effect(move || {
        let mut eval = document::eval(r#"
            if (window._ktv_remove_search_listener) {
                window._ktv_remove_search_listener();
            }
            const handler = (e) => {
                let tag = document.activeElement ? document.activeElement.tagName.toLowerCase() : '';
                if (tag === 'input' || tag === 'textarea') return;
                if (e.ctrlKey || e.metaKey || e.altKey) return;
                if (e.key === 'Escape') {
                    dioxus.send('ESC');
                    return;
                }
                if (e.key === ' ' || e.code === 'Space') {
                    e.preventDefault();
                    dioxus.send('SPACE');
                    return;
                }
                if (e.key === 'Backspace' || e.key === 'Delete') {
                    e.preventDefault();
                    dioxus.send('BACKSPACE');
                    return;
                }
                if (e.key === 'ArrowLeft') {
                    e.preventDefault();
                    dioxus.send('SEEK_REL:-5');
                    return;
                }
                if (e.key === 'ArrowRight') {
                    e.preventDefault();
                    dioxus.send('SEEK_REL:5');
                    return;
                }
                if (e.key.length === 1) {
                    dioxus.send('CHAR:' + e.key);
                }
            };
            window.addEventListener('keydown', handler);
            window._ktv_remove_search_listener = () => window.removeEventListener('keydown', handler);
        "#);

        spawn(async move {
            while let Ok(msg) = eval.recv::<String>().await {
                if let Some(ch) = msg.strip_prefix("CHAR:") {
                    let mut curr = search_query();
                    curr.push_str(ch);
                    search_query.set(curr);
                    active_tab.set(KtvTab::Catalog);
                } else if msg == "BACKSPACE" {
                    let mut curr = search_query();
                    curr.pop();
                    search_query.set(curr);
                    active_tab.set(KtvTab::Catalog);
                } else if msg == "ESC" {
                    search_query.set(String::new());
                } else if msg == "SPACE" {
                    let _ = document::eval(r#"
                        if (typeof window._ktv_toggle_playback === 'function') {
                            window._ktv_toggle_playback();
                        }
                    "#);
                } else if let Some(rel_str) = msg.strip_prefix("SEEK_REL:") {
                    if let Ok(delta) = rel_str.parse::<i64>() {
                        let js = format!(r#"
                            let cur = window._ktv_video_current_time;
                            if (!cur || cur <= 0) {{
                                let elapsed = (Date.now() - (window._ktv_video_mount_time || Date.now())) / 1000;
                                cur = Math.max(0, elapsed + (window._ktv_current_start_sec || 0));
                            }}
                            let target = Math.max(0, Math.floor(cur + ({delta})));
                            window._ktv_video_mount_time = Date.now();
                            window._ktv_current_start_sec = target;
                            window._ktv_video_current_time = target;
                            let iframe = document.getElementById('ktv-youtube-player');
                            if (iframe && iframe.contentWindow) {{
                                iframe.contentWindow.postMessage(JSON.stringify({{
                                    event: 'command',
                                    func: 'seekTo',
                                    args: [target, true]
                                }}), '*');
                            }}
                        "#);
                        let _ = document::eval(&js);
                    }
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

        // Check queue
        let mut q = queue();
        if !q.is_empty() {
            let next_item = q.remove(0);
            queue.set(q);
            current_song.set(Some(next_item));
            auto_dj_notice.set(None);
        } else {
            // Queue is empty: Trigger Auto-DJ wake_consume from katgpt anticipation set!
            let ant_set = anticipated_set();
            if let Some(rec) = anticipator().wake_consume(&ant_set) {
                let qid = next_queue_id();
                next_queue_id.set(qid + 1);

                auto_dj_notice.set(Some(format!(
                    "🧠 Auto-DJ: เพลงในคิวหมดแล้ว! เล่นต่ออัตโนมัติ: {} - {} ({})",
                    rec.song.title, rec.song.artist, rec.reason
                )));

                current_song.set(Some(QueueItem {
                    queue_id: qid,
                    song: rec.song,
                    key_shift: 0,
                    requester: "Smart Auto-DJ".to_string(),
                }));
            } else {
                current_song.set(None);
            }
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

    // Replay current song
    let handle_replay_song = move |_: ()| {
        if let Some(curr) = current_song() {
            song_started_at.set(js_sys::Date::now());
            current_song.set(None);
            current_song.set(Some(curr));
        }
    };

    // Play immediate song
    let handle_play_song = move |song: Song| {
        let qid = next_queue_id();
        next_queue_id.set(qid + 1);
        song_started_at.set(js_sys::Date::now());

        current_song.set(Some(QueueItem {
            queue_id: qid,
            song,
            key_shift: 0,
            requester: "Singer".to_string(),
        }));
    };

    // Add to queue
    let handle_queue_song = move |song: Song| {
        let qid = next_queue_id();
        next_queue_id.set(qid + 1);

        let new_item = QueueItem {
            queue_id: qid,
            song,
            key_shift: 0,
            requester: "Guest".to_string(),
        };

        if current_song().is_none() {
            song_started_at.set(js_sys::Date::now());
            current_song.set(Some(new_item));
        } else {
            let mut q = queue();
            q.push(new_item);
            queue.set(q);
        }
    };

    // Insert next in queue
    let handle_queue_next_song = move |song: Song| {
        let qid = next_queue_id();
        next_queue_id.set(qid + 1);

        let new_item = QueueItem {
            queue_id: qid,
            song,
            key_shift: 0,
            requester: "Priority".to_string(),
        };

        if current_song().is_none() {
            song_started_at.set(js_sys::Date::now());
            current_song.set(Some(new_item));
        } else {
            let mut q = queue();
            q.insert(0, new_item);
            queue.set(q);
        }
    };

    // Play by 5-digit code
    let handle_play_by_code = move |code: String| {
        if let Some(s) = catalog().iter().find(|s| s.code == code).cloned() {
            let qid = next_queue_id();
            next_queue_id.set(qid + 1);
            song_started_at.set(js_sys::Date::now());
            current_song.set(Some(QueueItem {
                queue_id: qid,
                song: s,
                key_shift: 0,
                requester: "Remote Code".to_string(),
            }));
        }
    };

    // Queue by 5-digit code
    let handle_queue_by_code = move |code: String| {
        if let Some(s) = catalog().iter().find(|s| s.code == code).cloned() {
            let qid = next_queue_id();
            next_queue_id.set(qid + 1);
            let item = QueueItem {
                queue_id: qid,
                song: s,
                key_shift: 0,
                requester: "Remote Code".to_string(),
            };
            if current_song().is_none() {
                song_started_at.set(js_sys::Date::now());
                current_song.set(Some(item));
            } else {
                let mut q = queue();
                q.push(item);
                queue.set(q);
            }
        }
    };

    // Key shift on current song
    let handle_key_change = move |delta: i32| {
        if let Some(mut curr) = current_song() {
            let new_key = (curr.key_shift + delta).clamp(-6, 6);
            curr.key_shift = new_key;
            current_song.set(Some(curr));
        }
    };

    // Reset key to original
    let handle_reset_key = move |_: ()| {
        if let Some(mut curr) = current_song() {
            curr.key_shift = 0;
            current_song.set(Some(curr));
        }
    };

    // Adjust key for a song in the queue
    let handle_adjust_item_key = move |(qid, delta): (u64, i32)| {
        let mut q = queue();
        if let Some(item) = q.iter_mut().find(|it| it.queue_id == qid) {
            item.key_shift = (item.key_shift + delta).clamp(-6, 6);
            queue.set(q);
        }
    };

    // Queue reordering
    let handle_move_up = move |idx: usize| {
        let mut q = queue();
        if idx > 0 && idx < q.len() {
            q.swap(idx, idx - 1);
            queue.set(q);
        }
    };

    let handle_move_down = move |idx: usize| {
        let mut q = queue();
        if idx + 1 < q.len() {
            q.swap(idx, idx + 1);
            queue.set(q);
        }
    };

    let handle_remove_queue = move |qid: u64| {
        let mut q = queue();
        q.retain(|it| it.queue_id != qid);
        queue.set(q);
    };

    let handle_clear_queue = move |_: ()| {
        queue.set(Vec::new());
    };

    // Add custom song from YouTube
    let handle_add_custom_song = move |(new_song, play_now): (Song, bool)| {
        let mut cat = catalog();
        cat.push(new_song.clone());
        catalog.set(cat);

        let qid = next_queue_id();
        next_queue_id.set(qid + 1);

        let item = QueueItem {
            queue_id: qid,
            song: new_song,
            key_shift: 0,
            requester: "YouTube Direct".to_string(),
        };

        if play_now || current_song().is_none() {
            song_started_at.set(js_sys::Date::now());
            current_song.set(Some(item));
        } else {
            let mut q = queue();
            q.push(item);
            queue.set(q);
        }
    };

    let current_key = current_song().map(|c| c.key_shift).unwrap_or(0);
    let ant_candidates = anticipated_set().candidates;
    let ant_commitment = anticipated_set().commitment_hash;

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div { class: "ktv-app-wrapper",
            // Header Bar
            Header {
                active_tab,
                queue_len: queue().len(),
                room_name: settings().room_name.clone(),
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
                        auto_skip_intro: settings().auto_skip_intro,
                        playback_speed: playback_speed(),
                        on_next_song: handle_next_song,
                        on_replay_song: handle_replay_song,
                        on_key_change: handle_key_change,
                        on_video_ended: handle_video_ended,
                    }
                }

                // Right / Tabbed Controller Panel
                section { class: "stage-control-side",
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
                                commitment_hash: ant_commitment,
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
