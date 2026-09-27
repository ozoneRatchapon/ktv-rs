use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use super::tip_qr::QrSvg;
use crate::browser;
use crate::room::{
    self, BoothMessage, BoothState, LinkStatus, PhoneCommand, RemoteConfig, RoomKey, ServerMessage, Socket,
    SocketEvent, CLOSE_REPLACED,
};
use crate::storage::{self, ROOM_KEY};
use crate::tip;

/// Reconnect waits double from this after each failed connection, up to `MAX_BACKOFF_MS`.
const MIN_BACKOFF_MS: i32 = 1_000;
const MAX_BACKOFF_MS: i32 = 30_000;

/// The booth's side of the phone remote, owned by the app for the whole session (not just the Keypad tab).
#[derive(Clone, Copy)]
pub struct RoomLink {
    pub status: ReadSignal<LinkStatus>,
    /// The page phones open (`None` on the host, where there is no origin).
    pub phone_url: Memo<Option<String>>,
    key: Signal<RoomKey>,
    socket: Signal<Option<Rc<Socket>>>,
}

impl RoomLink {
    /// A new room: phones on the old link are told to scan again, and the old key is forgotten.
    pub fn new_link(mut self) {
        if let Some(socket) = self.socket.take() {
            send(&socket, &BoothMessage::CloseRoom);
        }
        if let Some(key) = fresh_key() {
            storage::save(ROOM_KEY, &key);
            self.key.set(key);
        }
    }
}

/// While `config.enabled`: stay connected to the room (reconnecting with backoff), publish `state` whenever it
/// changes, and answer each phone command with what `on_command` did (`Ok`) or why not (`Err`).
pub fn use_room_link(
    config: Memo<RemoteConfig>,
    state: Memo<BoothState>,
    on_command: Callback<PhoneCommand, Result<String, String>>,
) -> RoomLink {
    let key = use_signal(|| {
        let saved = storage::load::<RoomKey>(ROOM_KEY).filter(room::is_valid_key);
        let key = saved.or_else(fresh_key).unwrap_or_else(|| RoomKey(String::new()));
        storage::save(ROOM_KEY, &key);
        key
    });
    let mut status = use_signal(LinkStatus::default);
    let mut socket = use_signal(|| None::<Rc<Socket>>);
    let phone_url = use_memo(move || {
        let origin = browser::page_origin()?;
        Some(room::phone_url(&origin, &room::room_of(&key.read())))
    });

    // Restarts on enable / disable and on a new key; dropping the old future drops (closes) its socket
    use_resource(move || {
        let enabled = config().enabled;
        let key = key();
        async move {
            socket.set(None);
            status.set(if enabled { LinkStatus::Connecting } else { LinkStatus::Off });
            let url = browser::page_origin().and_then(|origin| room::socket_url(&origin, &room::room_of(&key)));
            let (true, Some(url)) = (enabled, url) else { return };
            let mut backoff = MIN_BACKOFF_MS;
            loop {
                let (tx, mut events) = futures_channel::mpsc::unbounded();
                let Some(conn) = Socket::connect(&url, tx).map(Rc::new) else { return };
                let mut close_code = None;
                while let Some(event) = events.next().await {
                    match event {
                        SocketEvent::Open => {
                            backoff = MIN_BACKOFF_MS;
                            send(&conn, &BoothMessage::Hello { key: key.0.clone() });
                            status.set(LinkStatus::Online { phones: 0 });
                            // The state effect below sends the current state once the socket is set
                            socket.set(Some(conn.clone()));
                        }
                        SocketEvent::Message(text) => match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(ServerMessage::Phones { n }) => status.set(LinkStatus::Online { phones: n }),
                            Ok(ServerMessage::Cmd { from, cmd }) => {
                                let (ok, text) = match on_command.call(cmd) {
                                    Ok(text) => (true, text),
                                    Err(text) => (false, text),
                                };
                                send(&conn, &BoothMessage::Reply { to: from, ok, text });
                            }
                            Err(err) => dioxus::logger::tracing::warn!("room: unreadable message: {err}"),
                        },
                        SocketEvent::Closed(code) => {
                            close_code = Some(code);
                            break;
                        }
                    }
                }
                socket.set(None);
                if close_code == Some(CLOSE_REPLACED) {
                    status.set(LinkStatus::Replaced);
                    return;
                }
                status.set(LinkStatus::Connecting);
                browser::sleep_ms(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF_MS);
            }
        }
    });

    // Phones see every change of song, queue or permission
    use_effect(move || {
        let state = state();
        if let Some(conn) = socket() {
            send(&conn, &BoothMessage::State { state });
        }
    });

    RoomLink { status: status.into(), phone_url, key, socket }
}

fn fresh_key() -> Option<RoomKey> {
    let bytes = tip::reference_bytes()?;
    Some(room::key_from_bytes(&bytes))
}

fn send(socket: &Socket, msg: &BoothMessage) {
    if let Ok(text) = serde_json::to_string(msg) {
        socket.send(&text);
    }
}

/// Keypad tab: turn the phone remote on, show its QR, choose what phones may do, or make a new link.
#[component]
pub fn PhoneRemotePanel(
    config: RemoteConfig,
    status: LinkStatus,
    phone_url: Option<String>,
    on_config: EventHandler<RemoteConfig>,
    on_new_link: EventHandler<()>,
) -> Element {
    // Two taps: a stray tap must not disconnect every guest
    let mut confirm_new = use_signal(|| false);
    let qr = use_memo(use_reactive((&phone_url,), |(url,)| url.as_deref().and_then(tip::qr_path)));
    let status_text = match status {
        LinkStatus::Off => "Off: phones cannot reach this booth".to_string(),
        LinkStatus::Connecting => "Connecting…".to_string(),
        LinkStatus::Online { phones: 0 } => "Online: scan the code with a phone camera".to_string(),
        LinkStatus::Online { phones: 1 } => "Online · 1 phone connected".to_string(),
        LinkStatus::Online { phones } => format!("Online · {phones} phones connected"),
        LinkStatus::Replaced => "Another tab is the booth for this room now. Reload to take it back.".to_string(),
    };

    rsx! {
        div { class: "phone-remote", aria_label: "Phone remote",
            div { class: "phone-remote-head",
                div {
                    div { class: "setting-title", "Phone remote" }
                    div { class: "setting-sub", role: "status", "{status_text}" }
                }
                button {
                    class: if config.enabled { "toggle-btn active" } else { "toggle-btn" },
                    aria_pressed: "{config.enabled}",
                    onclick: move |_| on_config.call(RemoteConfig { enabled: !config.enabled, ..config }),
                    if config.enabled { "ON" } else { "OFF" }
                }
            }
            if config.enabled {
                if let (Some(url), Some(qr)) = (phone_url, qr()) {
                    div { class: "phone-remote-body",
                        a { class: "phone-remote-qr-link", href: "{url}", target: "_blank", rel: "noopener",
                            QrSvg { qr, class: "phone-remote-qr", label: "QR code: open the phone remote" }
                        }
                        div { class: "phone-remote-options",
                            label { class: "phone-remote-check",
                                input {
                                    r#type: "checkbox",
                                    id: "remote_allow_playback",
                                    name: "remote_allow_playback",
                                    checked: config.allow_playback,
                                    onchange: move |e| on_config.call(RemoteConfig { allow_playback: e.checked(), ..config }),
                                }
                                "Phones may skip, pause and replay (off: queue songs only)"
                            }
                            button {
                                class: "ctrl-btn action-btn",
                                onclick: move |_| {
                                    if confirm_new() {
                                        confirm_new.set(false);
                                        on_new_link.call(());
                                    } else {
                                        confirm_new.set(true);
                                    }
                                },
                                if confirm_new() { "Tap again: phones on the old code are disconnected" } else { "New link" }
                            }
                            span { class: "setting-sub",
                                "Song titles in the queue pass through this site's server to the phones; no names are kept."
                            }
                        }
                    }
                }
            }
        }
    }
}
