# Plan 001: KTV-RS roadmap (critical → sustainable → GOAT → Super GOAT)

Date: 2026-09-26 · Branch: `develop` · Status legend: `[ ]` todo, `[~]` in progress, `[x]` done, `[gate]` owner decision

## Verified current state (2026-09-26)
- Real: catalog (43 songs), type-to-search, queue, auto-DJ, scrubber, measured dual-player vocal switch (≤0.05s), click shield.
- **Placeholder (UI exists, no effect):**
  - Key transpose `-1/+1` only changes `key_shift` label (`main.rs:333`); YouTube iframe audio is cross-origin, cannot be pitch-shifted.
  - Score HUD shows hard-coded `94.8` / `C#4 Match` (`player.rs:375`).
  - Standby "MOBILE REMOTE / scan code" (`player.rs:596`) — no backend.
- **Doc/legal overclaims:** handover lists "Pitch Scoring" as delivered; Settings claims zero-tracking while embeds use `youtube.com/embed` (sets cookies).
- Large uncommitted diff (9 files, +346/−205) — only copy is the working tree.
- ~~Sync core is ~120 `window._ktv_*` JS globals inside a Rust string in `player.rs` — no tests, no lint.~~ Resolved 2026-09-26 (Phase 1).
- ~~Catalog + guide data hard-coded in `catalog.rs`~~ (now `assets/catalog.json`); no CI; no deploy target.

## Phase 0 — Critical (this week)
- [gate] Commit current work (listening test first). Loss risk grows with every change.
- [x] Honesty pass (2026-09-26): removed mock Score HUD + QR remote placeholder (markup + CSS); transpose tooltips say "label only"; handover list corrected.
- [x] Privacy (2026-09-26): both embeds on `youtube-nocookie.com` (postMessage sync verified, err ≈ −0.055s); Settings now says "0 first-party cookies" + discloses YouTube storage.
- [ ] Verify (not reproducible on demand — needs observation during listening test; watch `KtvSync.debug().guide_error` staying > 1s): does an ad pre-roll on the guide MV break sync? (monetized official MVs may play ads in embeds; `playerState`/`currentTime` during ads unknown). If yes: detect (state/duration mismatch) and hold guide, fall back to karaoke audio with a notice.
- [x] (2026-09-26, verified with a bogus guide id: fallback < 1s, button re-enables next song) Guide failure fallback: if the guide iframe errors (`onError` 100/101/150 = removed / embed disabled), auto-switch back to karaoke vocal and disable the button for that song.

## Phase 1 — Sustainable (next 2–4 weeks)
- [x] (2026-09-26) Extract sync JS to `assets/ktv_sync.js` (one module, one namespace object instead of globals); Rust keeps only `eval` bootstrap + `dioxus.send` bridge.
  - Core is `create_sync(env)` with all browser access injected; `install()` wires it once to `window.KtvSync` and rebinds the Rust channel on remount. Devtools: `KtvSync.debug()`.
  - Typed Rust bridge `src/sync.rs`: `SyncCommand` → JS, JS → `SyncEvent::parse`; frame ids shared as consts. ~120 `window._ktv_*` globals gone (only the search key listener's remains).
  - `set_start` effect hoisted out of the `match` arm in `player.rs` (a hook in a conditional branch changes hook order when switching Some ↔ None).
  - Fixed: report timestamps now use `null` sentinels (a report at `now() == 0` used to read as "no report").
  - Verified in browser (#10004): vocal on, scrubs (+0.03 / −0.05s warm), pause-scrub keeps guide paused, resume +0.03s, no console errors.
- [x] (2026-09-26) Unit-test the controller as a pure function `(target, guideNow, state) → action` with Node/Deno test in `tests/` (seek lead learning, intro hold, rate steps, paused scrub).
  - `node --test tests/ktv_sync.test.cjs` — 15 tests: `decide_guide` precedence, `next_rate` hysteresis, `learn_lead`/`parse_lead`, storage, intro hold/release, lead training + persistence, reseek, paused scrub, pause/resume, standby no-op, message routing, guide-error fallback, progress.
  - `cargo test --test sync_test` — event parsing, command JS, JS/Rust contract (method names, frame ids).
  - `deno lint assets/ktv_sync.js tests/ktv_sync.test.cjs` clean. Wire all three into CI (item below).
- [x] (2026-09-26) Cold first-play poisons the COLD lead: seeks issued while the guide is not warm (`is_warm`: state unknown/−1/0/5) no longer train any lead; the rate controller absorbs the one-off buffering error. `load_song` forgets the previous guide state so each new song's first start is cold. Browser-verified: cued first start landed +0.20s, COLD stayed 0.1; next warm (state 2) start trained 0.1→0.133. Tests in `tests/ktv_sync.test.cjs`.
- [x] (2026-09-26) Stale pause across songs: `load_song` clears a carried-over pause (new iframe autoplays) and emits `PAUSE_STATE:0`. Browser-verified: pause → Next Song plays with the Pause label.
- [x] (2026-09-26) Replay was a no-op (same-tick `None`→`Some` coalesced, iframe never remounted; also left a paused song paused): now `SyncCommand::Restart(start)` → core `restart()` seeks both players in place and resumes. Browser-verified playing, paused, and with original vocal (guide err +0.04s).
- [x] (2026-09-26) Replay start honours the player's per-song "Play Intro" toggle (lifted to `main` as `intro_skipped` signal, shared by player button and remote) instead of the global auto-skip setting; display jumps to the real start second instead of flashing 00:00. `Song::start_sec(skip_intro)` is the single source (DRY across player/main).
- [x] (2026-09-26) CSS: all 39 `var(--neon-*)` refs were undefined since the 275b752 redesign (numpad active, skip button, badges rendered without colour) → mapped to the `--accent-*` palette (cyan/amber as-is, purple→primary, pink→red); removed unused `.standby-pulse` + `@keyframes float`.
- [x] (2026-09-26) Data out of code: `assets/catalog.json` (43 songs + `guide`), embedded via `include_str!` and parsed once (`LazyLock`) — no startup fetch, malformed JSON fails `cargo test`. `catalog.rs` 576 → 19 lines. `tools/guide_align.py --write` stores successful fits into the song whose `youtube_id` is the karaoke id (atomic replace, byte-identical serde format; verified no-op write = identical file). New `tests/catalog_test.rs` checks unique id/code, known category, 11-char YouTube ids, intro < duration.
- [~] (2026-09-26, written + `actionlint` clean; first run pending push `[gate: owner commit/push]`) CI (GitHub Actions): `.github/workflows/ci.yml` — `cargo test --locked`, native + wasm clippy `-D warnings`, `node --test tests/ktv_sync.test.cjs`, `deno lint`, `py_compile` of tools, then `dx build --release --platform web` (dioxus-cli 0.7.10 via cargo-binstall, artifact upload). `.github/workflows/link-rot.yml` — weekly Mon 09:00 ICT `tools/link_check.py` (stdlib oEmbed check of all 86 karaoke + guide ids; 1.6s locally, all ok; failure path verified with a bogus id) opens or comments on one `link-rot` issue.
- [ ] Deploy static build (Cloudflare Pages or GitHub Pages) `[gate: owner picks host]`; add `deploy.sh`; tag releases via gitflow `develop → main`.
- [x] (2026-09-26) Persist settings + session in `localStorage` (`src/storage.rs`, keys `ktv.settings.v1` / `ktv.session.v1`): current song, queue, next queue id, Add-URL songs. On load the session is reconciled with the built-in catalog (fresh guide data, no id collisions, no duplicate custom songs); unreadable/old-schema values fall back to the demo session. Web-only app (no SSR) so it loads during init, not after hydration. `web-sys` is a wasm-only dep. Tests: `tests/storage_test.rs`. Browser-verified: queue/current/auto-skip/custom song survive reload; corrupt values → defaults.
- [x] Add-URL songs all got code `99999`. Now `catalog::upsert_custom` assigns the lowest free code in the reserved `CUSTOM_CODES` (90001-99999; built-ins stay below, enforced by a test); re-adding the same video reuses its entry and code. `Session::reconcile` renumbers colliding legacy `99999` codes (queue items too). Browser-verified on the release build: legacy pair -> 99999/90001, re-add keeps 90002, keypad 90001 queues the right song.
- [x] (2026-09-26) Add URL showed success even when all custom codes were used. `upsert_custom` now returns `Result<Song, CustomCodesExhausted>`; `CustomAdd` takes a returning `Callback` and shows a red `role="alert"` error (Thai, names the 90001-99999 range) and keeps the form filled, or on success shows the assigned keypad code. Re-adding an existing video still works when full. Test `test_upsert_custom_errors_when_codes_exhausted_but_readd_still_works`; browser-verified on the release build with 9999 injected custom songs (nothing queued on error; re-add of 90007 succeeded).
- [x] Release build was degraded (dx exits 0 anyway): (a) wasm-opt aborted on std's DWARF, so wasm shipped unoptimised (2.3 MB) -> `[profile.release] strip = "debuginfo"` -> 846 KB; (b) the esbuild fallback copied `app.js` with unbundled `./snippets/` imports -> **blank page**. Local cause: dx's cached `~/.dx/tools/esbuild-0.27.3` was x86_64 (no Rosetta); moved to `.x86_64.bak`, dx re-fetched arm64. CI build step now fails on any dx `ERROR` or leftover snippet import.

## Phase 2 — GOAT (1–2 months): real scoring
- [ ] Offline reference melody per song: extend `tools/` pipeline (already downloads guide audio) → vocal separation (demucs) → f0 tracking (pYIN/CREPE) → note track, mapped to karaoke time via existing `GuideTrack { offset, rate }`. Store as compact `assets/melody/<code>.bin`.
- [~] In-browser mic pitch: `getUserMedia` + AudioWorklet → Rust/wasm pitch detector (YIN/McLeod), latency-calibrated (reuse the learned-lead idea: tap test).
  - [x] (2026-09-26) Detector `src/pitch/` — McLeod Pitch Method, allocation-free per frame, rejects silence (RMS gate), noise (clarity) and out-of-range tones (above-range tones used to read an octave down; fixed + tested). `NoteReading` (nearest MIDI note + cents, `A4` naming). `tests/pitch_test.rs`: 44.1/48 kHz sines E2–C6 within 5 cents, strong-2nd-harmonic vowels without octave errors, amplitude independence, silence/noise/out-of-range/short frames.
  - [x] (2026-09-26) Capture `src/mic/` — Rust owns it via `web-sys` (frames reach wasm as `Float32Array`, no JSON): `assets/mic_worklet.js` (loaded from a Blob URL) posts 2048-sample frames every 1024 samples; RAII guards release the device on stop, unmount, or a half-finished start. Echo cancellation on (removes this tab's backing track), noise suppression/AGC off. Typed `MicError` (denied / no device / busy / unsupported). Native builds get a stub that returns `Unsupported`.
  - [x] (2026-09-26) `PitchMeter` in the player bar: "Mic: Off/On/Error" toggle + live note and cents, labelled display-only (no score). Browser-verified on the release build with an injected oscillator mic: 130.81→C3, 196→G3, 220→A3, 440→A4, 445→A4 +19¢, 1500 Hz→"—"; mic track `ended` after toggling off; no console errors.
  - [ ] Latency calibration (tap test) — only meaningful once the Score HUD compares against a timed reference melody; blocked on the melody item above (licensing `[gate]`).
  - [ ] Real-mic check on phone/tablet browsers (Safari, Android Chrome) — part of the owner listening test.
- [ ] Real Score HUD: note-lane view + per-phrase score; octave-tolerant matching.
- [ ] Repurpose **key transpose** honestly: shifts the *target melody* for scoring ("sing in your key"), not the backing audio.
- [x] (2026-09-26) Benchmarks auto-numbered in `bench/NNN_*`: `bench/001_pitch_detector.md` — native 78 µs/frame, wasm 277 µs/frame (1.3% of one core at 48 kHz), `simd128` 101 µs but not shipped (Safari < 16.4 cannot load SIMD wasm); release wasm 846 → 878 KB. Harness `benches/pitch_detector.rs` (`cargo bench`), wasm via `bench/run_wasi.mjs` (Node WASI).

## Phase 3 — Super GOAT (quarter+)
- [ ] Phone remote: QR → room URL; room state via Cloudflare Worker + Durable Object or WebRTC data channel. **Note:** DO bindings currently blocked by the versions-API `10013`/PUT `10021` issue — prefer WebRTC or KV-polling until resolved.
- [ ] Duets / multi-mic scoring, party leaderboard (local-only, PDPA-safe).
- [ ] Scale catalog: semi-automated guide search + alignment + coverage check, human approve step.
- [ ] Offline/booth mode: PWA shell, kiosk fullscreen, controller/keypad hardware input.

## Out of scope / constraints (by design)
- No re-hosting or downloading YouTube audio for playback (ToS). Offline analysis artifacts (melody note tracks) only; confirm licensing stance before public deploy `[gate]`.
- Backing-track pitch shift is impossible with iframe embeds; only achievable with licensed own audio.

## References
- Issue: `.issues/001_vocal_switch_offset_and_scrubber_controls.md`
- Handover: `.handovers/001_ktv_complete_system_handover.md`
- Alignment tool: `tools/guide_align.py`
