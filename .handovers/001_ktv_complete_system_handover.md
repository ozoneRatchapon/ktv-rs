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
- **Synchronized Vocal Switch (Time-Preserved with Intro Offset Compensation):** Toggling between Karaoke and Original singer MV preserves the singing moment seamlessly without restarting from 0. Each song has a measured `GuideTrack { offset_secs, rate }` (timed by ear with the in-app Guide Timing Tools, Issue 001 updates 2026-09-26; the old yt-dlp `tools/guide_align.py` was removed); the karaoke video stays visible (muted) while the guide MV, shown in its own pane beside it, is rate-controlled to stay within ~0.1s.
- **On-Screen Play / Pause Controls:** Spacebar shortcut and an interactive on-screen button with active visual indicator.
- **Auto-DJ (KatGPT-RS Sleep-Time Compute):** Recommendation engine anticipates next songs during playback and automatically fills empty queues. *Pitch scoring is not implemented* (the mock Score HUD was removed 2026-09-26; real scoring is Phase 2 of `.plans/001_roadmap.md`). Key transpose only records a per-song key label — YouTube embed audio is cross-origin and cannot be pitch-shifted.
- **Guide Failure Fallback:** If the guide MV errors (removed / private / embedding disabled), playback falls back to karaoke audio and the vocal button shows "Unavailable" for that song.
- **Developer Portfolio & Legal Transparency Disclosure:** Educational showcase attribution in Settings. The app sets no first-party cookies; embeds use YouTube's privacy-enhanced mode (`youtube-nocookie.com`), but YouTube may still store data once playback starts, so this is *not* zero-tracking. ToS / licensing stance is an open owner decision.

---

## 2. Where is the Plan, Code, and Tests
- **Core Architecture & App Root:** [`src/main.rs`](file:///Users/ozone/karaoke/src/main.rs)
- **Player & Scrubber Component:** [`src/components/player.rs`](file:///Users/ozone/karaoke/src/components/player.rs)
- **Player Sync Core (JS) & Rust Bridge:** [`assets/ktv_sync.js`](file:///Users/ozone/karaoke/assets/ktv_sync.js), [`src/sync.rs`](file:///Users/ozone/karaoke/src/sync.rs); tests `node --test tests/ktv_sync.test.cjs` and `cargo test --test sync_test`
- **Catalog & Video ID Verification:** [`assets/catalog.json`](file:///Users/ozone/karaoke/assets/catalog.json) (data, embedded at compile time by [`src/catalog.rs`](file:///Users/ozone/karaoke/src/catalog.rs); validated by `tests/catalog_test.rs`)
- **Add URL parsing:** [`src/youtube.rs`](file:///Users/ozone/karaoke/src/youtube.rs) `parse_video_id` (strict 11-char id; tests in `tests/youtube_test.rs`)
- **Persistence (settings + session in `localStorage`):** [`src/storage.rs`](file:///Users/ozone/karaoke/src/storage.rs); tests `cargo test --test storage_test`
- **CI & link-rot:** `.github/workflows/ci.yml`, `.github/workflows/link-rot.yml`; run the link check locally with `python3 tools/link_check.py`
- **Tuning score (reference-free):** [`src/score/`](file:///Users/ozone/karaoke/src/score/mod.rs) (`TuningScorer`, `cargo test --test score_test`), shown by `TuningBadge` in `pitch_meter.rs`, reset per take via `Player { take_started_at }`.
- **Deploy:** `wrangler.jsonc` (assets-only Worker `ktv-rs`), `deploy/_headers`, `tools/build_web.sh` (CI + deploy), `deploy.sh` (main only; `--dry-run` anywhere). See `docs/CLOUDFLARE_WORKERS_GUIDE.md`. Local prod preview: `tools/build_web.sh && npx wrangler@4.141.0 dev` (restart wrangler dev after a rebuild: the script recreates `dist/`). Note: `/opt/homebrew/bin/dx` is Deno's `dx`; use `~/.cargo/bin/dx`.
- **YouTube policy:** no overlays on players, guide MV shown in a visible pane only while it plays, native player pause mirrored in `ktv_sync.js`.
- **Mic pitch meter:** detector [`src/pitch/`](file:///Users/ozone/karaoke/src/pitch/mod.rs) (McLeod Pitch Method; `cargo test --test pitch_test`), capture [`src/mic/`](file:///Users/ozone/karaoke/src/mic/mod.rs) + worklet [`assets/mic_worklet.js`](file:///Users/ozone/karaoke/assets/mic_worklet.js), UI [`src/components/pitch_meter.rs`](file:///Users/ozone/karaoke/src/components/pitch_meter.rs). Benchmarks: `bench/NNN_*.md` (next free number), harness `cargo bench --bench pitch_detector`.
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
- Rust 1.82+ and `wasm32-unknown-unknown` target (`wasm32-wasip1` only for the wasm benchmark).
- Dioxus CLI: `cargo install dioxus-cli --version 0.7.1`

### 2. Running Local Dev Server
```bash
dx serve --platform web --port 8080
```
Open `http://localhost:8080/`.

Release bundle: `~/.cargo/bin/dx build --release --platform web` (output under `$CARGO_TARGET_DIR/dx/app/release/web/public`). dx exits 0 even when wasm-opt/esbuild fail, so scan its log for `ERROR`: an esbuild fallback leaves `./snippets/` imports unbundled and the page renders blank. If esbuild "fails to run", check `file ~/.dx/tools/esbuild-*/esbuild` is arm64 (an x86_64 copy was found once) and delete it so dx re-downloads. On reload, `dx serve` serves its last full build, not hot patches: test persistence/startup code against a fresh build.

Testing the mic without a microphone: navigate with an init script that replaces `navigator.mediaDevices.getUserMedia` with an `OscillatorNode` → `createMediaStreamDestination().stream` (sawtooth, gain 0.3), then set `osc.frequency.value` and read `.pitch-readout`. Real mics need https or localhost.

Timing a guide: Settings → Guide Timing Tools ON, play the song, *Play guide*, *Hear both: On*, nudge until the echo disappears; for a drifting MV *Mark in sync* early and ≥ 30 s later, then *Fit speed*. *Save* keeps it on this device (`localStorage` `ktv.guides.v1`); ship it by pasting *Copy JSON* into the song in `assets/catalog.json` and running `cargo test`. `window.KtvSync.debug()` shows `guide_offset`, `guide_rate`, `guide_error`, `monitor`.

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
