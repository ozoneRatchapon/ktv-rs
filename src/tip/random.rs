//! Fresh bytes for a tip `reference`. It only has to be unique (it is a lookup key, not a secret).

/// 32 bytes from `crypto.getRandomValues` in the browser.
#[cfg(target_arch = "wasm32")]
pub fn reference_bytes() -> Option<[u8; 32]> {
    let mut bytes = [0u8; 32];
    web_sys::window()?.crypto().ok()?.get_random_values_with_u8_array(&mut bytes).ok()?;
    Some(bytes)
}

/// Host builds (tests, desktop): clock + counter through splitmix64; unique per call, not cryptographic.
#[cfg(not(target_arch = "wasm32"))]
pub fn reference_bytes() -> Option<[u8; 32]> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_nanos();
    let mut state = (nanos as u64) ^ COUNTER.fetch_add(1, Ordering::Relaxed).rotate_left(32);
    let mut bytes = [0u8; 32];
    for chunk in bytes.chunks_exact_mut(8) {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        chunk.copy_from_slice(&(z ^ (z >> 31)).to_le_bytes());
    }
    Some(bytes)
}
