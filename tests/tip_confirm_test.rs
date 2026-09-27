//! Plan 003 S2: tip confirmation against real devnet RPC answers (tests/fixtures/tip, fetched 2026-09-28).

use app::tip::{
    format_usdc, parse_rpc_url, parse_signatures, parse_tip, signatures_request, transaction_request, RpcUrlError,
    SolanaCluster, TipCheckError, TipConfig, TipExpectation, ALLOWED_RPC_HOSTS,
};
use serde_json::Value;

const TRANSFER: &str = include_str!("fixtures/tip/devnet_usdc_transfer_with_memo.json");
const SIGNATURES: &str = include_str!("fixtures/tip/devnet_signatures.json");
const SIGNATURE: &str = "4Lvx1q3oAd1AuZmthNvZ2sY9E6sMtiTz1i3yWmRMKhrZhUdM8gZHJ6iy9yV7uWnUywDqVGKZAtje4xAvVZrH1jVo";
/// Keys in the fixture: one it carries (stands in for a reference), the payee and the payer (token account owners).
const CARRIED_KEY: &str = "GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb";
const PAYEE: &str = "75AjMdh7Gn1TLigfze541AVJGJ4TyqBEaRZk3pozfBza";
const PAYER: &str = "8sh86hmWL4ka7U44dFn3U72ZagLsAME4iRMwajfgR8QT";
const DEVNET_USDC: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

fn expect(recipient: &'static str) -> TipExpectation<'static> {
    TipExpectation { reference: CARRIED_KEY, recipient, mint: DEVNET_USDC }
}

fn edited(edit: impl FnOnce(&mut Value)) -> String {
    let mut tx: Value = serde_json::from_str(TRANSFER).expect("fixture is JSON");
    edit(&mut tx);
    tx.to_string()
}

#[test]
fn a_real_devnet_usdc_transfer_is_a_tip_for_the_payee() {
    let tip = parse_tip(SIGNATURE, TRANSFER, &expect(PAYEE)).expect("a tip");
    assert_eq!(tip.signature, SIGNATURE);
    // 378.885000 → 378.886000 USDC
    assert_eq!(tip.amount, 1_000);
    assert_eq!(tip.memo.as_deref(), Some("9c2ca23ac8536c58d717f0adc231f55a"));
}

#[test]
fn money_leaving_or_another_token_is_not_a_tip() {
    // The payer's balance went down
    assert_eq!(parse_tip(SIGNATURE, TRANSFER, &expect(PAYER)), Err(TipCheckError::NoPayment));
    let mainnet_mint = TipExpectation { mint: SolanaCluster::Mainnet.usdc_mint(), ..expect(PAYEE) };
    assert_eq!(parse_tip(SIGNATURE, TRANSFER, &mainnet_mint), Err(TipCheckError::NoPayment));
}

#[test]
fn a_transfer_without_the_song_reference_is_not_counted() {
    let other_song = TipExpectation { reference: "11111111111111111111111111111111", ..expect(PAYEE) };
    assert_eq!(parse_tip(SIGNATURE, TRANSFER, &other_song), Err(TipCheckError::WrongReference));
}

#[test]
fn failed_unindexed_and_error_answers() {
    let failed = edited(|tx| tx["result"]["meta"]["err"] = serde_json::json!({ "InstructionError": [2, "Custom"] }));
    assert_eq!(parse_tip(SIGNATURE, &failed, &expect(PAYEE)), Err(TipCheckError::Failed));

    let unindexed = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
    assert_eq!(parse_tip(SIGNATURE, unindexed, &expect(PAYEE)), Err(TipCheckError::NotFound));
    assert!(TipCheckError::NotFound.is_retryable());

    let limited = r#"{"jsonrpc":"2.0","error":{"code":-32429,"message":"rate limited"},"id":1}"#;
    assert_eq!(parse_tip(SIGNATURE, limited, &expect(PAYEE)), Err(TipCheckError::Rpc(-32429)));
    assert_eq!(parse_tip(SIGNATURE, "<html>", &expect(PAYEE)), Err(TipCheckError::Malformed));
    assert!(!TipCheckError::NoPayment.is_retryable());
}

#[test]
fn failed_transactions_are_dropped_from_the_signature_list() {
    let signatures = parse_signatures(SIGNATURES).expect("parses");
    assert_eq!(signatures.len(), 2, "2 of the 4 failed on-chain");
    assert!(signatures[0].starts_with("5YBeuHMmMz"));
    assert!(signatures[1].starts_with("hrQ9WgobCE"));
    assert_eq!(parse_signatures(r#"{"jsonrpc":"2.0","result":[],"id":1}"#), Ok(vec![]));
}

#[test]
fn requests_are_json_rpc_calls_at_confirmed_commitment() {
    let call: Value = serde_json::from_str(&signatures_request(CARRIED_KEY)).expect("JSON");
    assert_eq!(call["method"], "getSignaturesForAddress");
    assert_eq!(call["params"][0], CARRIED_KEY);
    assert_eq!(call["params"][1]["commitment"], "confirmed");

    let call: Value = serde_json::from_str(&transaction_request(SIGNATURE)).expect("JSON");
    assert_eq!(call["method"], "getTransaction");
    assert_eq!(call["params"][0], SIGNATURE);
    assert_eq!(call["params"][1]["encoding"], "jsonParsed");
    assert_eq!(call["params"][1]["maxSupportedTransactionVersion"], 0);
}

#[test]
fn rpc_urls_must_be_https_and_allowed_by_the_csp() {
    assert_eq!(parse_rpc_url("  "), Ok(None));
    let helius = "https://mainnet.helius-rpc.com/?api-key=abc";
    assert_eq!(parse_rpc_url(&format!(" {helius} ")), Ok(Some(helius.to_string())));
    assert_eq!(parse_rpc_url("http://api.devnet.solana.com"), Err(RpcUrlError::NotHttps));
    assert_eq!(parse_rpc_url("https://evil.example/rpc"), Err(RpcUrlError::HostNotAllowed));
    assert_eq!(parse_rpc_url("https://api.devnet.solana.com.evil.example"), Err(RpcUrlError::HostNotAllowed));
    assert_eq!(parse_rpc_url("https://api.devnet.solana.com:8899"), Err(RpcUrlError::HostNotAllowed));

    let headers = include_str!("../deploy/_headers");
    let connect_src = headers.split("connect-src").nth(1).and_then(|rest| rest.split(';').next()).expect("connect-src");
    for host in ALLOWED_RPC_HOSTS {
        assert!(connect_src.contains(&format!("https://{host}")), "CSP connect-src lacks {host}");
    }
}

#[test]
fn tips_are_checked_on_devnet_by_default_and_mainnet_needs_a_keyed_rpc() {
    let devnet = TipConfig::default();
    assert_eq!(devnet.rpc_endpoint().as_deref(), Some("https://api.devnet.solana.com"));

    let mut mainnet = TipConfig { cluster: SolanaCluster::Mainnet, ..TipConfig::default() };
    assert_eq!(mainnet.rpc_endpoint(), None);
    mainnet.rpc_url = "https://evil.example".to_string();
    assert_eq!(mainnet.rpc_endpoint(), None, "a blocked URL is never called");
    mainnet.rpc_url = "https://mainnet.helius-rpc.com/?api-key=abc".to_string();
    assert_eq!(mainnet.rpc_endpoint().as_deref(), Some("https://mainnet.helius-rpc.com/?api-key=abc"));
}

#[test]
fn usdc_amounts_show_cents_and_no_trailing_zeros() {
    assert_eq!(format_usdc(0), "0.00");
    assert_eq!(format_usdc(1_000), "0.001");
    assert_eq!(format_usdc(500_000), "0.50");
    assert_eq!(format_usdc(1_500_000), "1.50");
    assert_eq!(format_usdc(2_000_000), "2.00");
    assert_eq!(format_usdc(123_456_789), "123.456789");
}
