//! Plan 003 S3: what a tip asks for, read from its on-chain memo. The on-screen QR writes `ktv:<code>` (a tip for
//! the song playing); the phone request page writes `ktv:req:<code>` (play this song next).

use crate::catalog::keypad_code;
use crate::links::percent_encode;

use super::types::{ConfirmedTip, SolanaCluster};

const REQUEST_PREFIX: &str = "ktv:req:";

/// A request jumps the queue only with at least 0.10 USDC, so dust transfers cannot reorder a room.
pub const MIN_REQUEST_TIP: u64 = 100_000;

/// The memo a phone writes to request a song: `ktv:req:<code>`.
pub fn request_memo(code: &str) -> String {
    format!("{REQUEST_PREFIX}{code}")
}

/// What the booth does with a confirmed tip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TipAction {
    /// Thank the singer (garland toast).
    Garland,
    /// Queue this keypad code next, marked as a tip request.
    Request(String),
}

/// A request memo with a keypad code and at least [`MIN_REQUEST_TIP`] queues the song; anything else is a tip.
pub fn tip_action(tip: &ConfirmedTip) -> TipAction {
    let code = tip.memo.as_deref().and_then(|memo| memo.strip_prefix(REQUEST_PREFIX)).and_then(keypad_code);
    match code {
        Some(code) if tip.amount >= MIN_REQUEST_TIP => TipAction::Request(code.to_string()),
        _ => TipAction::Garland,
    }
}

/// The phone page a guest opens to request a song with a tip. Everything rides in the `#fragment`, so no server
/// (not even this one) sees the wallet or room in a request log.
pub fn request_page_url(origin: &str, wallet: &str, cluster: SolanaCluster, reference: &str, room: &str) -> String {
    let (cluster, room) = (cluster.slug(), percent_encode(room));
    format!("{origin}/request.html#to={wallet}&c={cluster}&ref={reference}&room={room}")
}
