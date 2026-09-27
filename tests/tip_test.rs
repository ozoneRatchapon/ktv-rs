use app::storage::decode;
use app::tip::{
    parse_wallet, qr_path, reference_bytes, reference_from_bytes, song_memo, transfer_url, SolanaCluster, TipConfig,
    TipRequest, WalletError, QUIET_ZONE,
};
use app::types::AppSettings;

/// A real mainnet address (the USDC mint itself): 32 bytes in base58.
const WALLET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn request(cluster: SolanaCluster) -> TipRequest {
    TipRequest {
        recipient: WALLET.to_string(),
        cluster,
        reference: reference_from_bytes(&[7u8; 32]),
        label: "VIP ROOM 07".to_string(),
        message: "Tip #12345".to_string(),
        memo: song_memo("12345"),
    }
}

#[test]
fn wallet_is_trimmed_and_must_be_a_32_byte_key() {
    assert_eq!(parse_wallet(&format!("  {WALLET}\n")), Ok(WALLET.to_string()));
    assert_eq!(parse_wallet("   "), Err(WalletError::Empty));
    // 0, O, I and l are not base58
    assert_eq!(parse_wallet("0OIl"), Err(WalletError::NotBase58));
    assert!(matches!(parse_wallet("abc"), Err(WalletError::WrongLength(_))));
    assert!(matches!(parse_wallet(&format!("{WALLET}{WALLET}")), Err(WalletError::WrongLength(_))));
}

#[test]
fn transfer_url_follows_the_solana_pay_spec() {
    let url = transfer_url(&request(SolanaCluster::Mainnet));
    let reference = reference_from_bytes(&[7u8; 32]);
    assert_eq!(
        url,
        format!(
            "solana:{WALLET}?spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&reference={reference}\
             &label=VIP%20ROOM%2007&message=Tip%20%2312345&memo=ktv%3A12345"
        )
    );
    // No amount: the guest chooses it in the wallet
    assert!(!url.contains("amount="));
}

#[test]
fn devnet_uses_the_devnet_usdc_mint() {
    let url = transfer_url(&request(SolanaCluster::Devnet));
    assert!(url.contains("spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"));
    assert_eq!(SolanaCluster::Devnet.toggled(), SolanaCluster::Mainnet);
    assert_eq!(SolanaCluster::Mainnet.toggled(), SolanaCluster::Devnet);
}

#[test]
fn thai_room_names_are_percent_encoded() {
    let mut req = request(SolanaCluster::Devnet);
    req.label = "ห้อง 1".to_string();
    let url = transfer_url(&req);
    assert!(url.contains("label=%E0%B8%AB%E0%B9%89%E0%B8%AD%E0%B8%87%201&"));
    assert!(url.is_ascii());
}

#[test]
fn references_are_32_byte_keys_and_unique_per_call() {
    let a = reference_bytes().expect("bytes");
    let b = reference_bytes().expect("bytes");
    assert_ne!(a, b);
    let reference = reference_from_bytes(&a);
    assert_eq!(parse_wallet(&reference).map(|r| r.len()), Ok(reference.len()));
}

#[test]
fn qr_path_draws_a_scannable_size_with_quiet_zone() {
    let url = transfer_url(&request(SolanaCluster::Mainnet));
    let qr = qr_path(&url).expect("fits in a QR code");
    // A tip URL (~220 bytes) at level L fits version 9 (53 modules); each side keeps its quiet zone
    let modules = qr.size - 2 * QUIET_ZONE;
    assert!((21..=53).contains(&modules), "{modules} modules");
    assert!(qr.d.starts_with(&format!("M{QUIET_ZONE} {QUIET_ZONE}h1v1h-1z")), "finder pattern corner is dark");
    assert_eq!(qr_path(&url), Some(qr), "deterministic");
}

#[test]
fn older_saved_settings_load_with_the_tip_qr_off() {
    let old = r#"{"default_intro_skip_secs":18,"auto_skip_intro":true,"volume":85,"room_name":"A","sound_fx_enabled":true}"#;
    let settings: AppSettings = decode(Some(old)).expect("old settings still load");
    assert_eq!(settings.tip, TipConfig::default());
    assert_eq!(settings.tip.cluster, SolanaCluster::Devnet);
    assert_eq!(parse_wallet(&settings.tip.wallet), Err(WalletError::Empty));
}
