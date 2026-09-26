# Issue 001: Video Focus Trap, Vocal Switch Offset Skew, and Playback Scrubbing Controls

## Status: Resolved
**Date:** 2026-09-25  
**Component:** `src/components/player.rs`, `src/main.rs`, `src/catalog.rs`, `src/types.rs`, `assets/main.css`

## Summary of Issues
1. **Focus Stealing & Keyboard Search Freeze:** Clicking anywhere on the YouTube video frame shifted browser OS focus into the cross-origin `<iframe>`. The top-level window lost keyboard focus, preventing users from using global Type-to-Search.
2. **Missing Scrubbing / Time Selection:** Because a protective click shield was applied over the entire video to prevent external navigation to YouTube.com, users were unable to click or drag the native YouTube seekbar to skip forwards or backwards.
3. **Vocal Switch Cue Mismatch (18s GMM Audio Start):** Toggling between Karaoke and Official MV resulted in lyrics playing out of sync. Deep RMS waveform analysis revealed that GMM Karaoke tracks feature a 13-second jingle bumper followed by 5 seconds of black screen/silence (RMS = 0.0) — music actually starts at second 18, not 13.
4. **Vocal Switch Restart Bug:** Switching back from Official MV to Karaoke reset playback to the beginning instead of resuming seamlessly from the elapsed point.
5. **Non-Functional Play/Pause:** The Play/Pause button and Spacebar sent `togglePlay` via postMessage, an invalid command not recognized by the YouTube IFrame API.

## Root Cause Analysis
- **Focus:** Cross-origin iframes capture all mouse events and steal document focus when clicked directly.
- **Scrubbing:** Blocking pointer events directly to the iframe without providing a custom KTV slider prevented user time seeking.
- **Offset & Intro:** Audio wave profiling proved the music starts at 18.0s (silence between 13.0s and 18.0s). The true offset formula is $\text{Offset} = \text{MV Intro} - 18\text{s}$.
- **Play/Pause Command:** YouTube IFrame Player API exclusively accepts `pauseVideo` and `playVideo`, discarding unrecognized methods like `togglePlay`.

## Solution & Implementation
1. **Full-Coverage Click Shield (`.video-click-shield-full`):** Transparent overlay div covers the iframe completely (`inset: 0; z-index: 10`), calling `_ktv_toggle_playback()` while refocusing `window.focus()`.
2. **Valid YouTube IFrame API Controls:** Replaced invalid `togglePlay` with `pauseVideo` and `playVideo` methods. Playback ticker freezes during pause without timestamp drift.
3. **Unified Pause State Event:** Registered `ktv-pause-state` CustomEvent, ensuring the on-screen button, click shield, and keyboard Spacebar stay 100% synchronized in real time.
4. **Calibrated 18s Vocal Offset:** Set `intro_skip_secs: 18` across all GMM tracks and calibrated `guide_offset_secs` ($\text{Offset} = \text{MV Intro} - 18$).
5. **KTV Interactive Timeline Scrubber & Arrow Keys:** Range slider and `ArrowLeft` / `ArrowRight` (-5s / +5s) seeking with `seekTo(target, true)`.

## Update 2026-09-26: Vocal switch was still out of sync — root causes & fix
The `Offset = MV Intro − 18s` rule and hand-tuned integer offsets were wrong. Measuring karaoke vs MV audio (chroma cross-correlation, `tools/guide_align.py`) showed:
- **Wrong offsets:** e.g. #10004 was `+4`, measured `−18.02`; #10025 `12` → `−3.15`; #10033 `−20` → `−12.10`. Blanket `−20` for COCKTAIL tracks was only right for some.
- **Guide == karaoke video:** #10007, #10019, #10021 pointed the guide at the karaoke video itself (no vocals). Replaced with official Topic/artist uploads.
- **Wrong MV version:** #10028 guide was an 11-min short-film MV (no audio match). Replaced with the official cut version.
- **Tempo drift:** #10031 MV runs 2.33% faster than karaoke — no constant offset can work.

Fix:
1. `Song.guide: Option<GuideTrack { video_id, offset_secs: f32, rate: f32 }>` — mapping `MV = offset + rate × karaoke`, all 14 values measured (fit RMS ≤ 0.08s).
2. Dual-player switch: karaoke video keeps playing muted (lyrics visible); hidden guide iframe plays the vocal.
3. Closed-loop sync every 250ms: guide nudged with playbackRate 0.9/0.95/1.05/1.1 (YouTube only allows 0.05 steps), re-seek only if error > 1s; learned seek lead (~0.3s); guide messages filtered out of karaoke time/end-of-song.
4. `tests/catalog_test.rs` guards guide ≠ karaoke and rate/offset ranges.

Browser-measured sync error after switch: 0.00–0.12s (settled ~0.03–0.08s), including #10031.

**Adding a song with a guide:** add the song to `assets/catalog.json`, then `python3 tools/guide_align.py --write <karaoke_id> <guide_id>` stores `offset_secs`/`rate` into its `guide` (2026-09-26: catalog moved out of `catalog.rs`). `null` = wrong guide video, entry left untouched. Run `cargo test` after.

## Update 2026-09-26 (b): Guides for the full catalog (43/43)
- Measured and added guides for the remaining 29 songs (all GMM 100xx + What The Duck 200xx). Preference: `- Topic` upload > artist/label official MV > official lyric upload; all oEmbed-embeddable (200).
- Every chosen pair: fit RMS ≤ 0.064s, rate 1.0 (except #10024 0.9995). Offsets cluster at −18s for GMM (karaoke intro card); #10018 −32.82, #10015 −13.02, #10023 −15.58. WTD `[Official Karaoke]` uploads share the master, so offsets are ~0 to +5s.
- **Edited MVs are a trap:** #20010 official MV (`pPa1d5cC8M4`) has a ~4.4s insert at ~140s; the old fit silently used only the first half. `tools/guide_align.py` now requires inliers to span ≥80% of the song (`coverage`), else `null`. Used the lyric upload `0wiPKB3MDsM` instead. All 14 earlier guides re-verified unchanged under the new guard.
- Outliers in other pairs are repeated choruses matching another chorus (offset returns to the fit afterwards) — benign.
- `tools/guide_align.py` download now falls back to HLS formats (`ba/91/92/93/94/b`) since dash audio intermittently 403s.
- Browser check: #10018 settles to 0.02s (0.22s transient when the guide leaves the intro hold), #20010 steady at −0.02s.

~~Remaining (optional): compensate start latency when the guide leaves the intro hold / after a main-player seek.~~ Done in (c).

## Update 2026-09-26 (c): Per-restart-kind seek lead (transients removed)
- The single learned lead (0.3s) was wrong in direction for two paths: the guide landed **ahead** (err = target − guide = −0.42s at intro release, −0.30s after a scrub), then took ~4s at 0.9/0.95× to converge. The earlier "0.22s late" note had the sign inverted.
- Restart latency depends on how the guide restarts, so `window._KTV_LEAD` now names three independently learned leads (`player.rs`), persisted in `localStorage`:
  - `RESEEK` (`_ktv_seek_lead`, default 0.3): guide-only re-seek while both play (error > 1s).
  - `COLD` (`_ktv_cold_lead`, default 0.1, learns ~0.077): guide starts from paused while karaoke runs — intro-hold release, vocal switch on.
  - `PAIR` (`_ktv_pair_lead`, default 0.05, learns ~0.02): karaoke and guide restart together — scrub/±10s, resume.
- Fixed along the way: scrubbing while paused no longer starts the guide.
- Browser (#10018, fresh `localStorage`): release −0.024s, scrub −0.034s / +0.002s, resume ≤0.016s — all from the first sample, no convergence tail. Paused scrub keeps guide paused. No console errors.

## Update 2026-09-26 (later): measuring without downloads
`tools/guide_align.py` downloaded karaoke and MV audio with yt-dlp, which YouTube's terms forbid, so it was removed (owner decision). New guides are timed by ear in the app: Settings → Guide Timing Tools (`src/timing/`, `src/components/guide_timing.rs`). *Hear both* plays the karaoke music under the guide so an offset is heard as an echo; *Mark in sync* early and late + *Fit speed* recovers `rate` for drifting MVs. The measured `guide` values already in `assets/catalog.json` stay. The edited-MV trap above still applies: if marks early and late disagree beyond a plausible speed (outside 0.9–1.1), Fit speed refuses; pick another upload.

## Update 2026-09-26 (focus without a shield)
The click shield (fix 1 above) was removed for YouTube policy, which brought back issue 1. Now a window `blur` listener in `main.rs` (next to the keydown listener, removed with it) checks on the next tick whether focus went into an iframe and, if so, blurs it and calls `window.focus()`. The click itself already reached the player, so its controls still work; only keystrokes come back to Type-to-Search.
