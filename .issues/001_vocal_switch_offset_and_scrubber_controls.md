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
