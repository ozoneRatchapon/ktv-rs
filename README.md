# KTV-RS: Modern Open-Source Karaoke Platform 🎤

> A high-performance, minimalist Web Karaoke player built 100% in **Rust** using **Dioxus 0.7.1 (Web / WASM)**. Designed for real-world party rooms and home setups with official Thai music feeds (`@gmmkaraoke`, `@whattheduckmusic`), automated intro bumper bypass, verified song catalog, dual-stream vocal toggles, and an intelligent Auto-DJ recommendation engine.

---

## Features

### 1. Direct Official Karaoke Streams
* Indexed official karaoke catalogs from **GMM Grammy** and **What The Duck**.
* Includes full verified collections for top artists like **COCKTAIL** (*คุกเข่า*, *เธอ*, *คู่ชีวิต*, *ดึงดัน*, *เธอทำให้ฉันเสียใจ*), **BOWKYLION** (*ที่คั่นหนังสือ*, *วาดไว้*), **Silly Fools** (*วัดใจ*), **Big Ass** (*เล่นของสูง*), etc.

### 2. Smart Platform Intro Skip
* Automatically skips introductory channel bumpers (e.g. GMM's 13s intro) directly to the music start, with an on-screen toggle to play the intro if desired.

### 3. Dual-Stream Vocal Switcher
* **Karaoke Mode**: Official backing track for singing.
* **Original Vocal Mode**: Instant switch to official artist MV to hear the original singer while preserving playback position.

### 4. 10-Key KTV Keypad Remote
* 5-digit quick code dialer (e.g. `#10026` for *คุกเข่า*).
* Real-time Key Transposition (`-2` to `+2` semitones) and tempo control (`0.9x` to `1.1x`).

### 5. Automated Next-Song Transition
* YouTube Iframe API event bridge detects video completion (`onStateChange: 0`) and transitions seamlessly to the next song in the queue.

### 6. Auto-DJ Recommendation Engine (`katgpt-rs` inspired)
* **Dwell Telemetry**: Records active singing duration per track.
* **Tropical $(\max, +)$ Semiring**: Hard-prunes songs skipped prematurely (< 20 seconds).
* **BLAKE3 State Commitment**: 32-byte cryptographic hashes for instantaneous reactive change detection.
* **Auto Fallback**: Automatically continues music playback with the top anticipated recommendation when the queue finishes.

---

## Getting Started

### Prerequisites
* Rust 1.85+
* Dioxus CLI 0.7.1:
  ```bash
  cargo install dioxus-cli --version 0.7.1
  ```

### Run Locally
```bash
dx serve --platform web --port 8080
```
Open [http://localhost:8080](http://localhost:8080) in your browser.

### Run Tests
```bash
cargo test --test recommendation_test
```

---

## Documentation Links

* [Architecture & Math Guide](file:///Users/ozone/karaoke/docs/ARCHITECTURE.md)
* [Cloudflare Workers & Pages Deployment Guide](file:///Users/ozone/karaoke/docs/CLOUDFLARE_WORKERS_GUIDE.md)
* [Developer Handover & Solana Integration Blueprint](file:///Users/ozone/karaoke/docs/DEVELOPER_GUIDE.md)

---

## License

MIT License. Embeds comply with YouTube API Terms of Service.
