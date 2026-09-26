# KTV-RS Developer & Agent Handover Guide 🛠️

> **Project Name Recommendation**: `ktv-rs` (or `karaoke-rs`)  
> **Tech Stack**: Pure Rust (100%), Dioxus 0.7.1 Web (WASM), Web Audio API, YouTube Iframe API Bridge, Tropical $(\max, +)$ Algebra Recommendation Engine.

---

## 📌 Executive Summary for Next Agent

This repository contains a full-featured, zero-latency Web KTV / Karaoke player designed for real party room use. The UI has been redesigned to be **clean, minimal, accessible, and free of emojis/neon clutter** (OG KTV aesthetic).

### Key Completed Modules:
1. **Catalog & Dual-Channel Playback** ([src/catalog.rs](file:///Users/ozone/karaoke/src/catalog.rs)):
   - Official Thai karaoke tracks from `@gmmkaraoke` and `@whattheduckmusic` (What The Duck).
   - Real, verified video IDs (e.g. COCKTAIL hits including `9aCUDQ8SPcA` - คุกเข่า, `Nlr1jPW6LZ4` - เธอ, `mMRkEh6Z6vE` - คู่ชีวิต, `7Bima2M9xy4` - ดึงดัน, `xmyqhV0f9Go` - ที่คั่นหนังสือ BOWKYLION, etc.).
   - Support for `guide_video_id` (original artist vocal track) toggle.
2. **Player & YouTube Bridge** ([src/components/player.rs](file:///Users/ozone/karaoke/src/components/player.rs)):
   - Bypasses platform intro bumpers (`&start={intro_skip_secs}`).
   - PostMessage bridge detecting clip completion (`onStateChange: 0`) for automatic advancing.
   - Live Pitch HUD & Guide Vocal toggle.
3. **Auto-DJ Substrate** ([src/recommendation.rs](file:///Users/ozone/karaoke/src/recommendation.rs)):
   - Sigmoid dwell-time tracking.
   - Tropical $(\max, +)$ semiring early-skip bottleneck pruner.
4. **10-Key KTV Keypad Remote** ([src/components/remote.rs](file:///Users/ozone/karaoke/src/components/remote.rs)):
   - 5-digit song code entry with instantaneous preview, play, queue, key transpose (`-2` to `+2`), and tempo adjustments (`0.9x` to `1.1x`).

---

## 🏃 Quick Start Commands

```bash
# 1. Run local development web server (Dioxus 0.7.1)
dx serve --platform web --port 8080

# 2. Run unit tests
cargo test --test recommendation_test

# 3. Check WASM compilation
cargo check --target wasm32-unknown-unknown
```

---

## 💡 Solana Integration Blueprint

### 1. Does Tipping Require KYC / Registration?
* **No**. In Web3/Solana, identity is permissionless:
  * **Sender**: Connects wallet (Phantom, Backpack, Solflare) or scans a Solana Pay QR code with any mobile wallet app.
  * **Singer / Performer**: Enters their Solana public key or `.sol` domain (Solana Name Service) in Room Settings.
  * Optionally, the sender can attach a **Display Name** and an **On-Screen Message** (e.g. "Tip 0.05 SOL: ขอเพลง คุกเข่า ให้พี่บอยหน่อยครับ!").

### 2. Live Chat & Special Request Tipping Mechanism
* **Priority Request via Tip (Song Request with Bounty)**:
  * Users can request a song from their phone and attach a small tip (e.g. $1–$5 in USDC or SOL).
  * The song is injected into the queue with a **Priority Request Badge** (`[★ TIP REQUEST]`).
  * If the singer accepts and sings the song, the escrowed tip is released to the singer's wallet upon completion.
* **Danmaku / Flying On-Screen Chat**:
  * Real-time WebSocket messages sent from mobile devices float across the TV screen (cheering, clapping sound effects, song requests).

---

## 🌐 GitHub Open-Source Recommendations

1. **Repository Name**:
   - `ktv-rs` (Recommended): Short, memorable, universally understood in Asia and globally.
   - `karaoke-rs`: Clean, highly searchable on GitHub and Google.
2. **Licensing**:
   - Use `MIT` or `Apache-2.0`.
   - Embeds comply with YouTube's Terms of Service (views and ad revenue flow directly to GMM and What The Duck).
