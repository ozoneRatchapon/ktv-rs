use dioxus::prelude::*;

use crate::tip::{self, ConfirmedTip, TipConfig, TipExpectation, TipRequest};
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
    let recipient = tip::parse_wallet(&config.wallet).ok();

    // S2: tips confirmed on-chain for this song. Restarts (and forgets) when the song, wallet or RPC changes.
    let mut tips = use_signal(Vec::<ConfirmedTip>::new);
    let watch_key = (reference(), recipient.clone(), config.rpc_endpoint(), config.cluster);
    use_resource(use_reactive((&watch_key,), move |((reference, recipient, rpc, cluster),)| async move {
        tips.set(Vec::new());
        let (Some(reference), Some(recipient), Some(rpc)) = (reference, recipient, rpc) else {
            return;
        };
        let expect = TipExpectation { reference: &reference, recipient: &recipient, mint: cluster.usdc_mint() };
        tip::watch_tips(&rpc, expect, |tip| tips.write().push(tip)).await;
    }));

    let url = match (recipient, reference()) {
        (Some(recipient), Some(reference)) => {
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
    let received = match tips.read().as_slice() {
        [] => String::new(),
        [one] => format!("✓ Tip received: {} USDC", tip::format_usdc(one.amount)),
        many => format!("✓ {} tips: {} USDC", many.len(), tip::format_usdc(many.iter().map(|t| t.amount).sum())),
    };

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
                // Live region: present (empty) before the first tip, so screen readers announce it
                span { class: "tip-received", role: "status", "{received}" }
                if is_test {
                    span { class: "tip-test-badge", "DEVNET · test money" }
                }
            }
        }
    }
}
