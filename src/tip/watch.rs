//! Plan 003 S2: poll the RPC for tips on the song on screen. Runs in the booth tab; the chain is the only
//! backend. Dropping the future (the song changed) stops it.

use std::collections::HashSet;

use crate::browser;

use super::rpc::{parse_signatures, parse_tip, signatures_request, transaction_request, TipExpectation};
use super::types::ConfirmedTip;

/// One `getSignaturesForAddress` per 5 s: a tip shows within seconds of the wallet's "sent", and a public RPC
/// (devnet: 100 requests per 10 s per IP) is never near its limit.
const POLL_MS: i32 = 5_000;
/// After failures (offline, HTTP 429) the wait doubles up to this.
const MAX_BACKOFF_MS: i32 = 60_000;
const RPC_TIMEOUT_MS: u32 = 10_000;

/// Call `on_tip` once per confirmed tip (deduped by signature), oldest first. Never returns.
pub async fn watch_tips(rpc: &str, expect: TipExpectation<'_>, mut on_tip: impl FnMut(ConfirmedTip)) {
    let mut settled = HashSet::new();
    let mut wait = POLL_MS;
    loop {
        wait = match poll_once(rpc, &expect, &mut settled, &mut on_tip).await {
            true => POLL_MS,
            false => (wait * 2).min(MAX_BACKOFF_MS),
        };
        browser::sleep_ms(wait).await;
    }
}

/// One round; `false` if the RPC could not be reached, so the caller backs off.
async fn poll_once(
    rpc: &str,
    expect: &TipExpectation<'_>,
    settled: &mut HashSet<String>,
    on_tip: &mut impl FnMut(ConfirmedTip),
) -> bool {
    let Some(body) = browser::post_json(rpc, &signatures_request(expect.reference), RPC_TIMEOUT_MS).await else {
        return false;
    };
    let Ok(signatures) = parse_signatures(&body) else {
        return false;
    };
    let fresh: Vec<String> = signatures.into_iter().rev().filter(|s| !settled.contains(s)).collect();
    for signature in fresh {
        let Some(body) = browser::post_json(rpc, &transaction_request(&signature), RPC_TIMEOUT_MS).await else {
            return false;
        };
        match parse_tip(&signature, &body, expect) {
            Ok(tip) => {
                settled.insert(signature);
                on_tip(tip);
            }
            // Not indexed yet: the next round asks again
            Err(err) if err.is_retryable() => {}
            Err(err) => {
                dioxus::logger::tracing::info!("tx {signature} is not a tip: {err:?}");
                settled.insert(signature);
            }
        }
    }
    true
}
