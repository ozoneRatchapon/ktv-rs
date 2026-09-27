use dioxus::prelude::*;

use crate::tip::{self, TipConfig, TipRequest};
use crate::types::QueueItem;

/// Plan 003 S1: a Solana Pay QR under the player. A guest scans it with a phone wallet and tips the singer
/// in USDC; the booth never holds a wallet. Drawn outside the video (YouTube: nothing in front of the player).
#[component]
pub fn TipQr(item: QueueItem, config: TipConfig, room_name: String) -> Element {
    // A fresh reference per queued song, so each tip is found on-chain by the song it was for (S2).
    // Hooks run before any early return, so their order never changes when the wallet is typed in.
    let queue_id = item.queue_id;
    let reference = use_memo(use_reactive((&queue_id,), |_| {
        tip::reference_bytes().map(|bytes| tip::reference_from_bytes(&bytes))
    }));
    let url = match (tip::parse_wallet(&config.wallet), reference()) {
        (Ok(recipient), Some(reference)) => {
            let code = item.song.code;
            Some(tip::transfer_url(&TipRequest {
                recipient,
                cluster: config.cluster,
                reference,
                label: room_name,
                message: format!("Tip #{code}"),
                memo: tip::song_memo(&code),
            }))
        }
        _ => None,
    };
    let qr = use_memo(use_reactive((&url,), |(url,)| url.as_deref().and_then(tip::qr_path)));
    let (Some(url), Some(qr)) = (url, qr()) else {
        return rsx! {};
    };
    let size = qr.size;
    let is_test = config.cluster == tip::SolanaCluster::Devnet;

    rsx! {
        div { class: "tip-strip", aria_label: "Tip the singer",
            a { class: "tip-qr-link", href: "{url}", title: "Open in a Solana wallet",
                svg {
                    class: "tip-qr",
                    view_box: "0 0 {size} {size}",
                    role: "img",
                    "aria-label": "Solana Pay QR code: tip the singer in USDC",
                    shape_rendering: "crispEdges",
                    rect { width: "{size}", height: "{size}", fill: "#fff" }
                    path { d: "{qr.d}", fill: "#000" }
                }
            }
            div { class: "tip-text",
                span { class: "tip-title", "Tip the singer" }
                span { class: "tip-sub", "Scan with Phantom or Solflare: USDC, you choose the amount" }
                if is_test {
                    span { class: "tip-test-badge", "DEVNET · test money" }
                }
            }
        }
    }
}
