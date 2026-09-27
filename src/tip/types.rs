use serde::{Deserialize, Serialize};

/// Which Solana cluster tips settle on. A phone wallet pays on the cluster it is set to; the mint in the QR must match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SolanaCluster {
    /// Test money: the default, so a demo never moves real funds by accident.
    #[default]
    Devnet,
    Mainnet,
}

impl SolanaCluster {
    /// Circle's USDC mint on this cluster.
    pub const fn usdc_mint(self) -> &'static str {
        match self {
            Self::Devnet => "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            Self::Mainnet => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Devnet => "Devnet (test USDC)",
            Self::Mainnet => "Mainnet (real USDC)",
        }
    }

    /// The free public RPC the booth checks tips with. Mainnet has none a browser may use (403).
    pub const fn public_rpc(self) -> Option<&'static str> {
        match self {
            Self::Devnet => Some("https://api.devnet.solana.com"),
            Self::Mainnet => None,
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Devnet => Self::Mainnet,
            Self::Mainnet => Self::Devnet,
        }
    }
}

/// Tip settings (saved with [`crate::types::AppSettings`]). An empty wallet turns the tip QR off.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TipConfig {
    /// The singer's (or host's) Solana address, base58.
    #[serde(default)]
    pub wallet: String,
    #[serde(default)]
    pub cluster: SolanaCluster,
    /// Optional RPC for tip checks (a keyed Helius URL); empty uses the cluster's public RPC.
    #[serde(default)]
    pub rpc_url: String,
}

impl TipConfig {
    /// Where to check tips: the typed RPC if it is allowed, else the cluster's public one. `None`: no check.
    pub fn rpc_endpoint(&self) -> Option<String> {
        match super::rpc::parse_rpc_url(&self.rpc_url) {
            Ok(Some(url)) => Some(url),
            _ => self.cluster.public_rpc().map(str::to_string),
        }
    }
}

/// Why a typed wallet address cannot receive tips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalletError {
    Empty,
    NotBase58,
    /// Decodes, but not to a 32-byte public key.
    WrongLength(usize),
}

impl WalletError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::Empty => "No wallet: the tip QR is off",
            Self::NotBase58 => "Not a Solana address (base58 letters and digits only)",
            Self::WrongLength(_) => "Not a Solana address (wrong length)",
        }
    }
}

/// One Solana Pay transfer request: a tip for the song on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TipRequest {
    pub recipient: String,
    pub cluster: SolanaCluster,
    /// Unique per song start; the payment is found on-chain by this key (plan 003 S2).
    pub reference: String,
    /// Shown by the wallet as the payee ("VIP ROOM 07").
    pub label: String,
    /// Shown by the wallet ("Tip #12345"); short, since every byte grows the QR code.
    pub message: String,
    /// Written on-chain with the transfer, so it is public: song code only, never a name.
    pub memo: String,
}

/// Why a typed RPC URL is not used (CSP `connect-src` would block it anyway).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcUrlError {
    NotHttps,
    HostNotAllowed,
}

impl RpcUrlError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::NotHttps => "RPC URL must start with https://",
            Self::HostNotAllowed => "Only api.devnet.solana.com or a Helius URL (devnet/mainnet.helius-rpc.com)",
        }
    }
}

/// A tip found on-chain for the song on screen (plan 003 S2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmedTip {
    pub signature: String,
    /// USDC base units (6 decimals) the singer's wallet gained.
    pub amount: u64,
    /// The memo the wallet wrote (`ktv:<code>` from the QR), if any.
    pub memo: Option<String>,
}

/// Why a transaction is not (yet) a tip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipCheckError {
    /// Not answerable yet (the RPC has not indexed it): ask again on the next poll.
    NotFound,
    /// The RPC answered with a JSON-RPC error code.
    Rpc(i64),
    Malformed,
    /// The transaction failed on-chain: no money moved.
    Failed,
    /// It does not carry this song's reference key.
    WrongReference,
    /// The singer's USDC balance did not go up (another token, another recipient, or a zero transfer).
    NoPayment,
}

impl TipCheckError {
    /// Worth asking about again later (vs. a settled "this is not a tip").
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::NotFound | Self::Rpc(_) | Self::Malformed)
    }
}

/// USDC decimals on every cluster.
pub const USDC_DECIMALS: u32 = 6;

/// `1500000` → `"1.50"`, `1000` → `"0.001"`: at least cents, no trailing zeros past them.
pub fn format_usdc(amount: u64) -> String {
    let scale = 10u64.pow(USDC_DECIMALS);
    let (whole, frac) = (amount / scale, amount % scale);
    let digits = format!("{frac:06}");
    let frac = digits.trim_end_matches('0');
    let frac = match frac.len() {
        0..=2 => &digits[..2],
        _ => frac,
    };
    format!("{whole}.{frac}")
}
