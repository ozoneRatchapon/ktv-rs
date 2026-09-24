use dioxus::prelude::*;

mod catalog;
mod components;
mod types;

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
use types::{AppSettings, KtvTab, QueueItem, Song};

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let settings = use_signal(AppSettings::default);
    let active_tab = use_signal(|| KtvTab::Catalog);
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

    // Next / Skip Song Handler
    let handle_next_song = move |_: ()| {
        let mut q = queue();
        if !q.is_empty() {
            let next_item = q.remove(0);
            queue.set(q);
            current_song.set(Some(next_item));
        } else {
            current_song.set(None);
        }
    };

    // Replay current song
    let handle_replay_song = move |_: ()| {
        if let Some(curr) = current_song() {
            // Trigger player refresh
            current_song.set(None);
            current_song.set(Some(curr));
        }
    };

    // Play immediate song
    let handle_play_song = move |song: Song| {
        let qid = next_queue_id();
        next_queue_id.set(qid + 1);

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
            current_song.set(Some(item));
        } else {
            let mut q = queue();
            q.push(item);
            queue.set(q);
        }
    };

    let current_key = current_song().map(|c| c.key_shift).unwrap_or(0);

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div { class: "ktv-app-wrapper",
            // Header Bar
            Header {
                active_tab,
                queue_len: queue().len(),
                room_name: settings().room_name.clone(),
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
                    }
                }

                // Right / Tabbed Controller Panel
                section { class: "stage-control-side",
                    match active_tab() {
                        KtvTab::Catalog => rsx! {
                            CatalogView {
                                catalog: catalog(),
                                on_play_song: handle_play_song,
                                on_queue_song: handle_queue_song,
                                on_queue_next_song: handle_queue_next_song,
                            }
                        },
                        KtvTab::Queue => rsx! {
                            QueueView {
                                queue: queue(),
                                current_item: current_song(),
                                on_skip: handle_next_song,
                                on_remove: handle_remove_queue,
                                on_move_up: handle_move_up,
                                on_move_down: handle_move_down,
                                on_clear_queue: handle_clear_queue,
                                on_adjust_item_key: handle_adjust_item_key,
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
