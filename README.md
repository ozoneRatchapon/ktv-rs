# NEON KTV - Pure Rust Dioxus Thai Karaoke Platform 🎤⚡

> A high-performance, cyberpunk-themed Web Karaoke platform written 100% in **Rust** using **Dioxus 0.7.1 (Web / WASM)**. Designed for modern KTV party rooms with official Thai music catalog (`@gmmkaraoke`), automated YouTube intro bumper skipping, real-time singer pitch evaluation, and an intelligent Auto-DJ recommendation engine inspired by `katgpt-rs`.

---

## 🌟 Key Features

### 1. ⚡ Smart Platform Intro Skip
* Traditional karaoke uploads (like GMM Official) feature an introductory channel bumper (approx. 13 seconds) before the music starts.
* NEON KTV automatically starts video playback directly at the musical onset (`start=13s`) while preserving an on-screen toggle to watch the intro if desired.

### 2. 🎙️ Dual-Stream Guide / Original Singer Vocal Switcher
* **Karaoke Mode (ดนตรีล้วน)**: Plays official instrumental backing track for singing.
* **Guide Vocal Mode (เสียงร้องจริง)**: Instant one-click toggle to play the official artist music video with full vocal performance, matching playback time without missing a beat.

### 3. 🎯 Real-Time Live Pitch & Mic Scorer (0–100 Points)
* Authentic Japanese KTV (Joysound / DAM style) real-time pitch feedback HUD.
* Compares singer frequency with musical melody notes (`C#4 PERFECT`), dynamic combo counters (`🔥 COMBO x18`), wave visualizers, and Rank evaluation (`RANK S / A / B`).

### 4. 🤖 Sleep-Time Auto-DJ Anticipation Engine (`katgpt-rs`)
* Inspired by the `katgpt-sleep` substrate:
  * **Dwell Telemetry**: Measures active listening duration per song.
  * **Sigmoid Affinity Gating**: Exponentially weights artist & genre affinity for completed songs.
  * **Tropical $(\max, +)$ Bottleneck Pruning**: Hard-prunes songs skipped in under 20 seconds using idempotent semiring algebraic masking.
  * **BLAKE3 State Commitment**: Cryptographically hashes queue state transitions for zero-overhead change detection.
  * **Auto-Advance Fallback**: When the queue finishes, Auto-DJ automatically serves the highest-ranked anticipated track.

### 5. ⏭️ Automated Clip-End Advancing
* Bi-directional YouTube Iframe API event bridge via Dioxus `document::eval`.
* Detects clip completion (`onStateChange: 0`) and automatically advances to the next song in queue with 0 user friction.

### 6. 🎛️ 10-Key KTV Remote & Pitch Transposition
* 5-digit quick song code dialer (e.g., `#10025` for *เล่นของสูง*).
* Real-time Key Transpose (`♭ -1` to `♯ +1` semitones) and playback tempo control (0.75x – 1.25x).

---

## 📂 Project Structure

```text
karaoke/
├── assets/
│   └── main.css             # Cyberpunk neon glassmorphic design system
├── docs/
│   ├── ARCHITECTURE.md      # Detailed system architecture & algorithms
│   └── CLOUDFLARE_WORKERS_GUIDE.md # Cloudflare workers-rs & Pages deployment
├── src/
│   ├── components/
│   │   ├── catalog_view.rs  # Categorized Songbook & quick search
│   │   ├── custom_add.rs    # Direct YouTube link/ID importer with offset
│   │   ├── player.rs        # YouTube Iframe, Guide Vocal & Pitch Scoring HUD
│   │   ├── queue_view.rs    # Real-time queue & Auto-DJ anticipation cards
│   │   ├── remote.rs        # 10-key digital numpad & key transposer
│   │   └── settings.rs      # Intro skip defaults & room personalization
│   ├── catalog.rs           # Curated GMM Thai songs + official MV pairs
│   ├── lib.rs               # Library root exposing recommendation engine
│   ├── main.rs              # App entry point, signal coordinator & router
│   ├── recommendation.rs    # Sleep-Time Anticipator (Tropical semiring + BLAKE3)
│   └── types.rs             # Strongly-typed models (Song, QueueItem, KtvTab)
└── tests/
    └── recommendation_test.rs # Verification tests for recommendation engine
```

---

## 🚀 Getting Started

### Prerequisites
* [Rust](https://www.rust-lang.org/) (latest stable, e.g. 1.85+)
* [Dioxus CLI](https://dioxuslabs.com/) (`dx` 0.7+)
  ```bash
  cargo install dioxus-cli --version 0.7.1
  ```

### Development Server
```bash
# Serve locally in browser
dx serve --platform web --port 8080
```
Open [http://localhost:8080](http://localhost:8080) to access the KTV room.

### Running Test Suite
```bash
cargo test --test recommendation_test
```

---

## ☁️ Cloudflare Deployment Architecture

NEON KTV is architected for zero-latency edge deployment using Cloudflare:
1. **Frontend**: Built to pure WebAssembly via `dx build --release --platform web` and served via **Cloudflare Pages**.
2. **Backend Room Sync**: Built using **Cloudflare Workers (`workers-rs`) + Durable Objects** to synchronize party room queue state, mobile QR remotes, and live pitch leaderboards across users.

*For complete deployment instructions, see [CLOUDFLARE_WORKERS_GUIDE.md](file:///Users/ozone/karaoke/docs/CLOUDFLARE_WORKERS_GUIDE.md).*
