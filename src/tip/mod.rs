//! Plan 003: tip the singer in USDC by scanning a Solana Pay QR on the screen (no wallet on the booth).

mod pay;
mod qr;
mod random;
mod types;

pub use pay::{parse_wallet, reference_from_bytes, song_memo, transfer_url};
pub use qr::{qr_path, QrPath, QUIET_ZONE};
pub use random::reference_bytes;
pub use types::{SolanaCluster, TipConfig, TipRequest, WalletError};
