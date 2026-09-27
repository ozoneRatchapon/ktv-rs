//! Plan 003 S2: find a tip on-chain by its `reference` key and check it paid the singer (Solana JSON-RPC).
//! Pure (request bodies in, response text out), so it is tested on the host against real devnet responses.

use serde::Deserialize;
use serde_json::json;

use super::types::{ConfirmedTip, RpcUrlError, TipCheckError};

/// Newest signatures first; a song rarely gets more than a handful of tips.
const SIGNATURE_LIMIT: usize = 20;

/// Hosts the booth may call for tip checks. Must match `connect-src` in `deploy/_headers` (a test checks it).
/// The public mainnet RPC answers browsers with 403, so mainnet needs a keyed Helius URL.
pub const ALLOWED_RPC_HOSTS: [&str; 3] = ["api.devnet.solana.com", "devnet.helius-rpc.com", "mainnet.helius-rpc.com"];

/// Check a typed RPC URL: empty means the cluster's public default. Returns the URL to store.
pub fn parse_rpc_url(input: &str) -> Result<Option<String>, RpcUrlError> {
    let url = input.trim();
    if url.is_empty() {
        return Ok(None);
    }
    let rest = url.strip_prefix("https://").ok_or(RpcUrlError::NotHttps)?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    match ALLOWED_RPC_HOSTS.contains(&host) {
        true => Ok(Some(url.to_string())),
        false => Err(RpcUrlError::HostNotAllowed),
    }
}

/// `getSignaturesForAddress(reference)`: every transaction that carried the song's reference key.
pub fn signatures_request(reference: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [reference, { "limit": SIGNATURE_LIMIT, "commitment": "confirmed" }],
    })
    .to_string()
}

/// `getTransaction(signature)` with parsed token balances (v0 transactions included).
pub fn transaction_request(signature: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [signature, { "encoding": "jsonParsed", "maxSupportedTransactionVersion": 0, "commitment": "confirmed" }],
    })
    .to_string()
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i64,
}

#[derive(Deserialize)]
struct SignatureInfo {
    signature: String,
    err: Option<serde_json::Value>,
}

/// Signatures of transactions that succeeded, newest first. Failed ones moved no money.
pub fn parse_signatures(body: &str) -> Result<Vec<String>, TipCheckError> {
    let infos: Vec<SignatureInfo> = rpc_result(body)?.ok_or(TipCheckError::Malformed)?;
    Ok(infos.into_iter().filter(|info| info.err.is_none()).map(|info| info.signature).collect())
}

#[derive(Deserialize)]
struct Transaction {
    meta: Option<Meta>,
    transaction: TransactionBody,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    err: Option<serde_json::Value>,
    #[serde(default)]
    pre_token_balances: Vec<TokenBalance>,
    #[serde(default)]
    post_token_balances: Vec<TokenBalance>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenBalance {
    mint: String,
    owner: Option<String>,
    ui_token_amount: UiTokenAmount,
}

#[derive(Deserialize)]
struct UiTokenAmount {
    /// Raw base units as a decimal string (u64 does not fit a JSON number safely).
    amount: String,
}

#[derive(Deserialize)]
struct TransactionBody {
    message: Message,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    account_keys: Vec<AccountKey>,
    #[serde(default)]
    instructions: Vec<Instruction>,
}

#[derive(Deserialize)]
struct AccountKey {
    pubkey: String,
}

#[derive(Deserialize)]
struct Instruction {
    program: Option<String>,
    parsed: Option<serde_json::Value>,
}

/// What a tip for this song must look like.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TipExpectation<'a> {
    pub reference: &'a str,
    pub recipient: &'a str,
    pub mint: &'a str,
}

/// Check one `getTransaction` answer, the same way Solana Pay's `validateTransfer` does: it succeeded,
/// carries the reference, and the recipient's balance of the mint went up. The rise is the tip.
pub fn parse_tip(signature: &str, body: &str, expect: &TipExpectation) -> Result<ConfirmedTip, TipCheckError> {
    let tx: Transaction = rpc_result(body)?.ok_or(TipCheckError::NotFound)?;
    let meta = tx.meta.ok_or(TipCheckError::Malformed)?;
    if meta.err.is_some() {
        return Err(TipCheckError::Failed);
    }
    let message = tx.transaction.message;
    if !message.account_keys.iter().any(|key| key.pubkey == expect.reference) {
        return Err(TipCheckError::WrongReference);
    }
    let held = |balances: &[TokenBalance]| -> Result<u128, TipCheckError> {
        balances
            .iter()
            .filter(|b| b.mint == expect.mint && b.owner.as_deref() == Some(expect.recipient))
            .map(|b| b.ui_token_amount.amount.parse::<u128>().map_err(|_| TipCheckError::Malformed))
            .sum()
    };
    let (before, after) = (held(&meta.pre_token_balances)?, held(&meta.post_token_balances)?);
    let amount = after.checked_sub(before).filter(|rise| *rise > 0).ok_or(TipCheckError::NoPayment)?;
    let amount = u64::try_from(amount).map_err(|_| TipCheckError::Malformed)?;
    let memo = message
        .instructions
        .iter()
        .find(|ix| ix.program.as_deref() == Some("spl-memo"))
        .and_then(|ix| ix.parsed.as_ref()?.as_str().map(str::to_string));
    Ok(ConfirmedTip { signature: signature.to_string(), amount, memo })
}

fn rpc_result<T: for<'de> Deserialize<'de>>(body: &str) -> Result<Option<T>, TipCheckError> {
    let response: RpcResponse<T> = serde_json::from_str(body).map_err(|_| TipCheckError::Malformed)?;
    match response.error {
        Some(RpcError { code }) => Err(TipCheckError::Rpc(code)),
        None => Ok(response.result),
    }
}
