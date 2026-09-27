//! The booth's WebSocket to its room. Events arrive on a channel; dropping the socket detaches the handlers
//! and closes it, so a cancelled connection loop never leaves a socket or a closure behind.

use futures_channel::mpsc::UnboundedSender;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

use super::types::SocketEvent;

pub struct Socket {
    ws: WebSocket,
    _on_open: Closure<dyn FnMut(Event)>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
}

impl Socket {
    /// Start connecting; `Open`, each text `Message`, then one `Closed` arrive on `events`. `None` if the URL is refused.
    pub fn connect(url: &str, events: UnboundedSender<SocketEvent>) -> Option<Self> {
        let ws = WebSocket::new(url).ok()?;
        let tx = events.clone();
        let on_open = Closure::<dyn FnMut(Event)>::new(move |_| {
            let _ = tx.unbounded_send(SocketEvent::Open);
        });
        let tx = events.clone();
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Some(text) = event.data().as_string() {
                let _ = tx.unbounded_send(SocketEvent::Message(text));
            }
        });
        // An error is always followed by a close event, so close alone ends the connection
        let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |event: CloseEvent| {
            let _ = events.unbounded_send(SocketEvent::Closed(event.code()));
        });
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        Some(Self { ws, _on_open: on_open, _on_message: on_message, _on_close: on_close })
    }

    /// Best effort: `false` if the socket is not open.
    pub fn send(&self, text: &str) -> bool {
        self.ws.ready_state() == WebSocket::OPEN && self.ws.send_with_str(text).is_ok()
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.ws.set_onopen(None);
        self.ws.set_onmessage(None);
        self.ws.set_onclose(None);
        let _ = self.ws.close_with_code(1000);
    }
}
