# Handover 001: KTV-RS Pure Rust Web Karaoke Platform Complete System Handover

**Project:** KTV-RS (Thai Karaoke & Official MV Web Platform)  
**Repository:** [https://github.com/ozoneRatchapon/ktv-rs](https://github.com/ozoneRatchapon/ktv-rs)  
**Branches:** `main`, `develop` (Gitflow compliant, fully synchronized)  
**Date:** 2026-09-25

---

## 1. What Happened
We built and perfected **KTV-RS**, an authentic, high-performance Web Karaoke platform written in pure Rust (Dioxus 0.7.1 WebAssembly). The system streams official Thai karaoke backing tracks (GMM Grammy, What The Duck) alongside genuine singer Official MVs for vocal guidance.

### Key Capabilities Delivered:
- **Type-to-Search (OG KTV Booth Style):** Global keystroke detection allows users to type song titles, artist names, or 5-digit song codes anywhere on the keyboard without needing to pre-focus the search bar. Backspace and Delete are fully supported for English and Thai scripts.
- **Full Video Click Shield:** Protects the screen against accidental redirection to external YouTube links, ensuring kiosk/booth safety while preserving parent window focus.
- **KTV Timeline Scrubber & Quick Jumps:** Drag-and-drop timeline slider (`min: 0`, `max: duration`) showing formatted `mm:ss` timestamps, with quick `-10s` and `+10s` jump buttons and `ArrowLeft` / `ArrowRight` (-5s / +5s) keyboard navigation.
- **Synchronized Vocal Switch (Time-Preserved with Intro Offset Compensation):** Toggling between Karaoke and Original singer MV preserves the singing moment seamlessly without restarting from 0. Each song has a calibrated `guide_offset_secs` accounting for GMM channel bumpers vs MV story intros.
- **On-Screen Play / Pause Controls:** Spacebar shortcut and an interactive on-screen button with active visual indicator.
- **Pitch Scoring & Auto-DJ (KatGPT-RS Sleep-Time Compute):** Autonomous recommendation engine anticipates next songs during playback and automatically fills empty queues.
- **Developer Portfolio & Legal Transparency Disclosure:** Clear educational showcase attribution in Settings compliant with YouTube Embed API ToS (Section 3.2), Fair Use, and zero-tracking privacy (PDPA / GDPR ready).

---

## 2. Where is the Plan, Code, and Tests
- **Core Architecture & App Root:** [`src/main.rs`](file:///Users/ozone/karaoke/src/main.rs)
- **Player & Scrubber Component:** [`src/components/player.rs`](file:///Users/ozone/karaoke/src/components/player.rs)
- **Catalog & Video ID Verification:** [`src/catalog.rs`](file:///Users/ozone/karaoke/src/catalog.rs)
- **Data Models & Types:** [`src/types.rs`](file:///Users/ozone/karaoke/src/types.rs)
- **Sleep-Time Recommendation Engine:** [`src/recommendation.rs`](file:///Users/ozone/karaoke/src/recommendation.rs)
- **UI Design System & Aesthetics:** [`assets/main.css`](file:///Users/ozone/karaoke/assets/main.css)
- **Developer Transparency & Legal Card:** [`src/components/settings.rs`](file:///Users/ozone/karaoke/src/components/settings.rs)
- **Unit Tests:** [`tests/recommendation_test.rs`](file:///Users/ozone/karaoke/tests/recommendation_test.rs)
- **End-to-End Browser Video Artifacts:**
  - Full player verification: `ktv_player_verified_1790270117397.png`
  - Play/Pause toggle and 18s offset sync: `ktv_player_final_1790271031727.png`
  - Browser test recording: `pause_play_offset_verification`

---

## 3. Reflection: Struggles & Solutions
| Problem Faced | Root Cause | How It Was Solved |
| :--- | :--- | :--- |
| **Search typing stopped working after clicking video** | Clicking inside YouTube's iframe transferred browser OS focus to `youtube.com`. Window `keydown` stopped firing. | Added a transparent full-coverage click shield (`.video-click-shield-full`) with `inset: 0; z-index: 10`. Clicks invoke `_ktv_toggle_playback()` and immediately refocus `window.focus()`. |
| **Play/Pause button and Spacebar did not work** | Sent `togglePlay` via postMessage, which is not a valid YouTube IFrame API command. | Replaced with valid `pauseVideo` and `playVideo` methods via `_ktv_toggle_playback()`, freezing the timestamp ticker and dispatching `ktv-pause-state` to keep UI state synchronized. |
| **Vocal switch audio was 5 seconds off** | Thought GMM songs started at 13s bumper end. RMS waveform profiling revealed 5 seconds of black screen/silence (RMS = 0.0) — music actually starts at 18.0s! | Calibrated `intro_skip_secs: 18` across all GMM tracks and used $\text{Offset} = \text{MV Intro} - 18\text{s}$ ($\text{MV Time} = \text{Karaoke Time} + \text{Offset}$). |
| **Vocal switch restarted playback from 00:00** | Raw MV seconds fell below the 18s Karaoke bumper when switching back, triggering initial intro state. | Used offset-compensated continuous time formula, persisting active timestamps across iframe switches. |
| **Could not scrub/seek audio position** | The protective shield covered YouTube's seekbar. | Built custom KTV Timeline Scrubber with slider, `-10s` / `+10s` buttons, and `ArrowLeft` / `ArrowRight` (-5s / +5s) shortcuts using `seekTo(target, true)`. |

---

## 4. Issues Reference
- Ref: [Issue 001: Video Focus Trap, Vocal Switch Offset Skew, and Playback Scrubbing Controls](file:///Users/ozone/karaoke/.issues/001_vocal_switch_offset_and_scrubber_controls.md)

---

## 5. How to Develop & Test
### 1. Prerequisites
- Rust 1.80+ and `wasm32-unknown-unknown` target.
- Dioxus CLI: `cargo install dioxus-cli --version 0.7.1`

### 2. Running Local Dev Server
```bash
dx serve --platform web --port 8080
```
Open `http://localhost:8080/`.

### 3. Running Unit Tests
```bash
cargo test -p app --test recommendation_test
```

### 4. Running Linting & Diagnostics
```bash
cargo clippy --fix --allow-dirty --quiet
```

### 5. Gitflow Deployment
- `develop`: Ongoing feature integration.
- `main`: Production-ready releases.
- Push commands: `git push origin develop && git push origin main`.
