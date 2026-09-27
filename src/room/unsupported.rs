//! Host builds (tests, desktop): no room socket.

use futures_channel::mpsc::UnboundedSender;

use super::types::SocketEvent;

pub struct Socket;

impl Socket {
    pub fn connect(_url: &str, _events: UnboundedSender<SocketEvent>) -> Option<Self> {
        None
    }

    pub fn send(&self, _text: &str) -> bool {
        false
    }
}
