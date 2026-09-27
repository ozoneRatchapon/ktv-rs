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
