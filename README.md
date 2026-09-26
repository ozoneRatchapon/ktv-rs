# KTV-RS: Modern Open-Source Karaoke Platform 🎤

> A high-performance, minimalist Web Karaoke player built 100% in **Rust** using **Dioxus 0.7.1 (Web / WASM)**. Designed for real-world party rooms and home setups with official Thai music feeds (`@gmmkaraoke`, `@whattheduckmusic`), automated intro bumper bypass, verified song catalog, dual-stream vocal toggles, and an intelligent Auto-DJ recommendation engine.

[![Build & Test](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Dioxus](https://img.shields.io/badge/Dioxus-0.7.1-blue.svg)](https://dioxuslabs.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Zero-Tracking](https://img.shields.io/badge/Privacy-Zero--Tracking-brightgreen.svg)](#privacy--legal-compliance)

---

## Key Features

### 1. Direct Official Karaoke Streams
* Indexed official karaoke catalogs from **GMM Grammy** and **What The Duck**.
* Includes full verified collections for top artists like **COCKTAIL** (*คุกเข่า*, *เธอ*, *คู่ชีวิต*, *ดึงดัน*, *เธอทำให้ฉันเสียใจ*), **BOWKYLION** (*ที่คั่นหนังสือ*, *วาดไว้*), **Silly Fools** (*วัดใจ*), **Big Ass** (*เล่นของสูง*), etc.

### 2. Unobstructed YouTube Players
* Nothing is drawn over either YouTube player (YouTube Required Minimum Functionality: no overlays in front of an embedded player); banners and badges live below the video.
* Pause/play from the YouTube player's own controls is mirrored into the app, so the guide and the timer stay in step.
* Clicking inside a video still leaves Type-to-Search working: the app takes keyboard focus back from the player right after the click.

### 3. KTV Timeline Scrubber & Quick Jumps
* Custom HTML5 timeline scrubber slider showing formatted `current_time` and `total_time` (`mm:ss`).
* Dedicated on-screen **`-10s`** and **`+10s`** jump buttons for repeating tricky vocal passages or skipping guitar solos.
* Keyboard arrow navigation: **`ArrowLeft`** (-5s) and **`ArrowRight`** (+5s).

### 4. Synchronized Vocal Switcher with Intro Offset Compensation
* **Karaoke Mode**: Official backing track with lyrics.
* **Original Vocal Mode**: Instant switch to the official artist MV, shown beside (or, on narrow screens, below) the karaoke video, to hear the singer while preserving song progress. The MV plays only while visible; it is never used as a hidden audio source.
* **Offset Compensation**: Each song's `guide` maps karaoke time to MV time (`MV = offset_secs + rate × karaoke`), so the switch lands on the same lyric without restarting from 0.
* **Guide Timing Tools** (Settings → Guide Timing Tools): line up an original-vocal MV by ear with the embedded players only (no audio download). Nudge ±1 s / ±0.1 s while *Hear both* plays the karaoke music under the guide (misalignment is heard as an echo); for an MV that drifts, *Mark in sync* early and late and *Fit speed*. *Save* keeps the timing on this device; *Copy JSON* gives the `guide` entry for `assets/catalog.json`.

### 5. OG KTV "Type-to-Search" (Global Keystroke Capture)
* Type any song title, artist, or 5-digit code anywhere on your keyboard to instantly filter the catalog.
* Native support for `Backspace`, `Delete`, and `Escape` for both English and Thai scripts.

### 6. 10-Key KTV Keypad Remote & On-Screen Play/Pause
* 5-digit quick code dialer (e.g. `#10026` for *คุกเข่า*).
* Real-time Key Transposition (`-6` to `+6` semitones) and tempo control (`0.9x` to `1.1x`).
* Dedicated **Play / Pause** toggle button with active state styling.

### 7. Auto-DJ Recommendation Engine (`katgpt-rs` inspired)
* **Dwell Telemetry**: Records active singing duration per track.
* **Tropical $(\max, +)$ Semiring**: Hard-prunes songs skipped prematurely (< 20 seconds).
* **Auto Fallback**: Automatically continues music playback with the top anticipated recommendation when the queue finishes.

---

## Keyboard Controls

| Key | Action |
| :--- | :--- |
| **Any Character / Number** | Type-to-Search across the catalog |
| **`Backspace` / `Delete`** | Delete character from search query |
| **`Escape`** | Clear search query |
| **`Space`** | Toggle Play / Pause |
| **`ArrowLeft`** | Jump backward 5 seconds |
| **`ArrowRight`** | Jump forward 5 seconds |
| **`?`** | Show / hide the shortcut list (also the **?** button in the header; open on a first visit) |

---

## Getting Started

### Prerequisites
* Rust 1.80+ (`wasm32-unknown-unknown` target)
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
cargo test -p app --test recommendation_test
```

### Run Linting
```bash
cargo clippy --fix --allow-dirty --quiet
```

---

## Privacy & Legal Compliance

* **100% Client-Side WebAssembly**: No media or copyrighted files are stored on our servers.
* **YouTube Embed Compliance**: Standard YouTube embedded players only, following the [YouTube API Developer Policies](https://developers.google.com/youtube/terms/developer-policies) and [Required Minimum Functionality](https://developers.google.com/youtube/terms/required-minimum-functionality): no overlays, no hidden or background playback, no audio separation, no downloading; players stay at least 200x200. All ad views, watch time, and royalties go directly to the respective record labels and artists.
* **Zero Tracking**: No tracking cookies, analytics pixels, or personal data collection (GDPR & PDPA compliant). Fonts (Kanit, Outfit; SIL OFL 1.1) are self-hosted from `public/fonts`, so the app itself makes no third-party requests; only the YouTube player frames (youtube-nocookie.com) and thumbnails (i.ytimg.com) load from Google.

---

## Documentation Links

* [Architecture & Math Guide](file:///Users/ozone/karaoke/docs/ARCHITECTURE.md)
* [Cloudflare Workers & Pages Deployment Guide](file:///Users/ozone/karaoke/docs/CLOUDFLARE_WORKERS_GUIDE.md)
* [Developer Handover & Solana Integration Blueprint](file:///Users/ozone/karaoke/docs/DEVELOPER_GUIDE.md)
* [System Handover 001](file:///Users/ozone/karaoke/.handovers/001_ktv_complete_system_handover.md)
* [Issue 001: Video Focus Trap & Vocal Offset Fix](file:///Users/ozone/karaoke/.issues/001_vocal_switch_offset_and_scrubber_controls.md)

---

## License

MIT License.
