use dioxus::prelude::*;

use crate::browser;
use crate::tip::{self, ConfirmedTip, QrPath, TipConfig, TipExpectation, TipRequest};
use crate::types::QueueItem;

/// Plan 003 S1: a Solana Pay QR under the player. A guest scans it with a phone wallet and tips the singer
/// in USDC; the booth never holds a wallet. Drawn outside the video (YouTube: nothing in front of the player).
/// S3: beside it, a camera QR for the phone request page (`request_page`), so a guest can pay to pick a song.
#[component]
pub fn TipQr(
    item: QueueItem,
    config: TipConfig,
    room_name: String,
    /// The room's request-page link (`None`: no request QR).
    request_page: Option<String>,
    on_tip: EventHandler<ConfirmedTip>,
) -> Element {
    // A fresh reference per queued song, so each tip is found on-chain by the song it was for (S2).
    // Hooks run before any early return, so their order never changes when the wallet is typed in.
    let queue_id = item.queue_id;
    let reference = use_memo(use_reactive((&queue_id,), |_| {
        tip::reference_bytes().map(|bytes| tip::reference_from_bytes(&bytes))
    }));
    let tips = use_tip_watch(reference(), &config, on_tip);

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
    let request_qr = use_memo(use_reactive((&request_page,), |(page,)| page.as_deref().and_then(tip::qr_path)));
    let (Some(url), Some(qr)) = (url, qr()) else {
        return rsx! {};
    };
    let is_test = config.cluster == tip::SolanaCluster::Devnet;
    let received = match tips.read().as_slice() {
        [] => String::new(),
        [one] => format!("✓ Tip received: {} USDC", tip::format_usdc(one.amount)),
        many => format!("✓ {} tips: {} USDC", many.len(), tip::format_usdc(many.iter().map(|t| t.amount).sum())),
    };

    rsx! {
        div { class: "tip-strip", aria_label: "Tip the singer",
            a { class: "tip-qr-link", href: "{url}", title: "Open in a Solana wallet",
                QrSvg { qr, class: "tip-qr", label: "Solana Pay QR code: tip the singer in USDC" }
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
            if let (Some(page), Some(request_qr)) = (request_page, request_qr()) {
                a { class: "tip-request", href: "{page}", target: "_blank", rel: "noopener",
                    QrSvg { qr: request_qr, class: "tip-request-qr", label: "QR code: request a song with a tip" }
                    span { class: "tip-sub", "Request a song: scan with your camera" }
                }
            }
        }
    }
}

/// S3: watch the room's request reference for the whole session (songs come and go; the room QR stays).
/// Renders nothing.
#[component]
pub fn TipRequestWatch(config: TipConfig, reference: String, on_tip: EventHandler<ConfirmedTip>) -> Element {
    use_tip_watch(Some(reference), &config, on_tip);
    rsx! {}
}

/// The request page link for this booth, if the wallet is valid and the page has an origin (not on the host).
pub fn request_page(config: &TipConfig, reference: &str, room_name: &str) -> Option<String> {
    let wallet = tip::parse_wallet(&config.wallet).ok()?;
    let origin = browser::page_origin()?;
    Some(tip::request_page_url(&origin, &wallet, config.cluster, reference, room_name))
}

#[component]
fn QrSvg(qr: QrPath, class: &'static str, label: &'static str) -> Element {
    let size = qr.size;
    rsx! {
        svg {
            class,
            view_box: "0 0 {size} {size}",
            role: "img",
            "aria-label": label,
            shape_rendering: "crispEdges",
            rect { width: "{size}", height: "{size}", fill: "#fff" }
            path { d: "{qr.d}", fill: "#000" }
        }
    }
}

/// S2: tips confirmed on-chain for `reference`, each also sent to `on_tip`. Restarts (and forgets) when the
/// reference, wallet, RPC or cluster changes; a wallet or RPC that is not set means no polling.
fn use_tip_watch(
    reference: Option<String>,
    config: &TipConfig,
    on_tip: EventHandler<ConfirmedTip>,
) -> Signal<Vec<ConfirmedTip>> {
    let mut tips = use_signal(Vec::<ConfirmedTip>::new);
    let watch_key = (reference, tip::parse_wallet(&config.wallet).ok(), config.rpc_endpoint(), config.cluster);
    use_resource(use_reactive((&watch_key,), move |((reference, recipient, rpc, cluster),)| async move {
        tips.set(Vec::new());
        let (Some(reference), Some(recipient), Some(rpc)) = (reference, recipient, rpc) else {
            return;
        };
        let expect = TipExpectation { reference: &reference, recipient: &recipient, mint: cluster.usdc_mint() };
        tip::watch_tips(&rpc, expect, |tip| {
            on_tip.call(tip.clone());
            tips.write().push(tip);
        })
        .await;
    }));
    tips
}
