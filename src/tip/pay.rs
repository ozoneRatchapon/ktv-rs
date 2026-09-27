//! Solana Pay transfer-request URLs (<https://docs.solanapay.com/spec>). Pure, so it is tested on the host.

use crate::links::percent_encode;

use super::types::{TipRequest, WalletError};

/// A public key is 32 bytes; base58 spells it in 32 to 44 characters.
const PUBKEY_LEN: usize = 32;

/// Trim and check a typed address; returns the address to store.
pub fn parse_wallet(input: &str) -> Result<String, WalletError> {
    let wallet = input.trim();
    if wallet.is_empty() {
        return Err(WalletError::Empty);
    }
    let bytes = bs58::decode(wallet).into_vec().map_err(|_| WalletError::NotBase58)?;
    match bytes.len() {
        PUBKEY_LEN => Ok(wallet.to_string()),
        len => Err(WalletError::WrongLength(len)),
    }
}

/// A `reference` key from 32 random bytes (any 32 bytes form a valid reference; it need not be on the curve).
pub fn reference_from_bytes(bytes: &[u8; PUBKEY_LEN]) -> String {
    bs58::encode(bytes).into_string()
}

/// The on-chain memo for a tip on a song: `ktv:<code>`. Public forever, so it carries no personal data.
pub fn song_memo(code: &str) -> String {
    format!("ktv:{code}")
}

/// `solana:<recipient>?spl-token=…&reference=…&label=…&message=…&memo=…`. No `amount`: the guest picks it in the wallet.
pub fn transfer_url(req: &TipRequest) -> String {
    let TipRequest { recipient, cluster, reference, label, message, memo } = req;
    let mint = cluster.usdc_mint();
    let (label, message, memo) = (percent_encode(label), percent_encode(message), percent_encode(memo));
    format!("solana:{recipient}?spl-token={mint}&reference={reference}&label={label}&message={message}&memo={memo}")
}
