//! Plan 003: tip the singer in USDC by scanning a Solana Pay QR on the screen (no wallet on the booth).

mod memo;
mod pay;
mod qr;
mod random;
mod rpc;
mod types;
mod watch;

pub use memo::{request_memo, request_page_url, tip_action, TipAction, MIN_REQUEST_TIP};
pub use pay::{parse_wallet, reference_from_bytes, song_memo, transfer_url};
pub use qr::{qr_path, QrPath, QUIET_ZONE};
pub use random::reference_bytes;
pub use rpc::{
    parse_rpc_url, parse_signatures, parse_tip, signatures_request, transaction_request, TipExpectation,
    ALLOWED_RPC_HOSTS,
};
pub use watch::watch_tips;
pub use types::{
    format_usdc, ConfirmedTip, RpcUrlError, SolanaCluster, TipCheckError, TipConfig, TipRequest, WalletError,
    USDC_DECIMALS,
};
