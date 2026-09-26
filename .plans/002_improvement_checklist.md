# 002 — Improvement checklist (2026-09-27)

Worked top to bottom. Each finished item: tests + clippy clean, browser-verified, pushed to `develop`, CI green, then released (owner's standing approval: deploy whenever an update is ready). Record details in `.plans/001_roadmap.md`; tick here with the commit/version.

Legend: **(checked)** = confirmed in code on 2026-09-27; `gated:` = waits on owner/device/external.

## Release now
- [x] R1. (v0.3.0, Worker `4b6f9408`, prod-verified) Release the 4 fixes already on `develop` (focus reclaim, favicon + input ids, narrow-screen layout, Revert wording, CI on ubuntu-26.04) as v0.3.0.

## Wave 1 — small, touches every user
- [x] 15. `<html>` has no `lang` **(checked)** → `lang="th"` (screen readers, Thai line breaking, font choice). Done: custom `index.html` with `lang="en"` (UI language) + meta description/theme-color; song titles/artists carry `lang="th"` (88 elements on the catalog page).
- [x] 16. UI language mix: mostly English with a few Thai messages **(checked)** → one UI language, consistent wording (TH/EN toggle later if wanted). Done: English UI; the 10 Thai messages (Add URL, Keypad, Auto-DJ, recommendation reasons) translated; app name unified to KTV-RS (was "NEON KTV" in the page title and docs).
- [x] 18. Keyboard shortcut help (`?`) + first-visit hint. Done: `components/shortcuts.rs` panel at the top of the control column (never over a player), `?` key + header **?** button toggle it, open on every visit until closed once (`AppSettings.seen_shortcuts`, serde default; storage test).

## Wave 2 — safety net before big work
- [x] 29. E2E browser tests in `tests/` (headless Chrome over CDP, no new deps): focus reclaim, type-to-search, Revert, no clipped controls at 375–1280 px; run in CI against the release build. Done: `tests/e2e/{cdp,app.test}.mjs` (9 tests, ~5 s; fresh browser context per test; fails on console errors and CSP violations). Negative check: dropping the control-row wrap fails 600/375 px naming Pause/Fullscreen/Replay/Next. CI build job serves `dist/` with `wrangler dev` and runs them.
- [x] 26. Split `main.rs` (590 lines **(checked)**) handlers/state into modules. Done: `src/booth/{mod,types,ops}.rs` — pure `Booth { current, queue, next_queue_id }` with `Placement { Now, Next, Back }` / `Requester` enums replaces 5 copies of the enqueue logic; `main.rs` holds one `Signal<Booth>` + memo slices (609 → 387 lines). `tests/booth_test.rs` (7) + e2e queue and Auto-DJ tests. Auto-DJ picks are read before advancing so it never replays the finished song.
- [x] 27. Move the keydown/blur listener out of the `window._ktv_remove_search_listener` global into `ktv_sync.js` / typed commands. Done: `assets/ktv_keys.js` (pure `key_action`, install-once + rebind like the sync core, also ignores `select`/contenteditable) + `src/keys.rs` `KeyAction::parse`; `tests/ktv_keys.test.cjs` (5) and `tests/keys_test.rs` (3, incl. JS↔Rust message contract).
- [x] 28. Review `recommendation.rs` blake3 `commitment_hash` **(checked)**: keep only if it earns its place. Done: removed — nothing ever read it (docs claimed UI equality checks; false), only a "BLAKE3: xxxx" badge. Dropped the hash, set/anticipator `version`, the badge CSS and the `blake3` dependency. wasm size unchanged (971 KB), so item 20 needs its own look.

## Wave 3 — singing core (Phase 2)
The score today is tuning precision (held notes vs the semitone grid); there is no melody to compare against.
- [x] 6. Real score HUD + end-of-song result screen. Done: `PitchMeter` reports each finished take (`score::TakeResult`, only if a held note was judged); result card at the top of the control column (score, verdict, song, notes, average ¢); Queue tab "Recent scores" (last 20, `ktv.scores.v1`). Tests: 3 in `score_test.rs`; e2e with an oscillator mic (score ≥ 90, card names the finished song, history survives reload).
- [x] 9. Noise gate so room noise / speakers don't score: adaptive to the room's noise floor (fixed `min_rms` 0.01 today). Done: `pitch::NoiseGate` — 1 s room check when the mic opens (median RMS, so a cough doesn't count), then a fixed gate at 2× the room, clamped 0.01–0.1 RMS. Deliberately not continuously adaptive: a creeping floor would shut out a singer who never pauses. UI shows "Room check…"; the Tuning explanation says so. Tests: 5 in `pitch_test.rs`; e2e plays a room-level tone during the check and shows a louder-than-0.01 but under-2×-room tone is ignored. Real-room tuning of the ratio: with the gated device tests (item 3).
- [ ] 7. Target pitch line. `gated:` needs per-song melody data. Extracting it from YouTube audio breaks YouTube's terms and published melodies are copyrighted, so the source is an owner decision (options: hand-entered/licensed MIDI, or a reference learned from the singer's own best take).
- [ ] 5. Honest key transpose (shift target melody, not audio). Blocked by 7 (with no melody, key shift changes nothing the app measures).
- [ ] 8. Mic latency calibration. Blocked by 7 (only matters when pitch is compared to a melody in time).
- [ ] 10. Duet mode. `gated:` needs two mic devices to test; blocked by 7 for per-part scoring.

- [x] 6b. Queue tab "End Song" button ("Simulate video ending") is a debug control shipped to users: hide it behind the Guide Timing Tools setting or remove. Done: shown only with Guide Timing Tools on (it records a natural finish, which would skew Auto-DJ).

## Wave 4 — content & search
- [x] 14. Thai-aware search: tone marks, romanised input ("rak mai wai"), typo tolerance. Done: `src/search/` — `normalize` (drops Thai tone marks/thanthakhat, spaces, punctuation, case), `aliases` on `Song` (24 songs, from the official GMM video titles via oEmbed, `tools/title_aliases.py`), Kedmanee ⇄ QWERTY `retype` used only when nothing matches as typed (UI says which query it used). Tests: `tests/search_test.rs` (7), e2e alias + wrong-layout type-to-search. General typo tolerance (edit distance) not done: with 43 songs the three rules cover the common misses; revisit with a bigger catalog.
- [x] 13. Favourites, recently sung, history (localStorage). Done: `src/picks/` (`Picks` + typed `Shelf` enum replacing the category string; `catalog::CATEGORIES` const), `ktv.picks.v1`. ☆/★ toggle on song cards; ★ Favourites / Recently sung chips (a song counts after 30 s on stage, newest first, 20 kept). Score history (item 6) covers mic takes. Tests: `tests/picks_test.rs` (4), e2e shelves.
- [x] 12. Community guide timing submissions (Copy JSON → PR template / issue form). Done: `.github/ISSUE_TEMPLATE/{guide_timing,song_request}.yml`; **Share on GitHub** in the timing panel opens the guide-timing form pre-filled (`timing::share_url`, UTF-8 percent-encoding; plain link, no background request). Test in `timing_test.rs` + e2e.
- [ ] 11. Grow the catalog beyond 43 songs **(checked)**. `gated:` which songs is an owner/curation call (official uploads only; duration + intro read from the player). Tooling ready: `docs/ADDING_SONGS.md`, Song request form, `tools/title_aliases.py`, `tools/link_check.py`, `catalog_test`.

## Wave 5 — performance
- [ ] 20. wasm 968 KB, was 846 KB **(checked)** → find the growth, try `wasm-opt -Oz`.
- [ ] 21. Dead CSS in `main.css` (2,226 lines **(checked)**).
- [ ] 22. Lighthouse / Core Web Vitals on prod.
- [ ] 19. Preload the next song's video for faster song changes.

## Wave 6 — accessibility
- [ ] 30. Visible focus, keyboard navigation of song cards, contrast audit.
- [ ] 31. `prefers-reduced-motion`; announce song changes (`aria-live`).

## Wave 7 — security & privacy
- [ ] 23. Remove CSP `'unsafe-eval'` if Dioxus allows.
- [ ] 24. Versioned `localStorage` migrations (no data loss on schema change).
- [ ] 25. Privacy page backing the "Zero Tracking" claim.

## Wave 8 — platform (Phase 3)
- [ ] 17. TV / booth mode (10-foot UI).
- [ ] 32. PWA: installable, offline app shell.
- [ ] 33. Phone remote. `gated:` Durable Objects blocked by Cloudflare `10013`/`10021`; recheck the versions API or pick another transport.

## Wave 9 — ops & docs
- [ ] 34. Deploy from CI with an approval environment.
- [ ] 35. CHANGELOG / release notes per tag.
- [ ] 36. Uptime check on the prod URL.
- [ ] 37. README screenshots + "adding songs" guide. (Adding-songs guide done: `docs/ADDING_SONGS.md`; screenshots still to do.)

## Gated (not worked until the owner/device says so)
- [ ] 1. Custom domain. `gated:` owner deferred (costs money).
- [ ] 3. Real mic test on iPhone Safari / Android Chrome. `gated:` needs devices.
- [ ] 4. YouTube pre-roll ads vs sync. `gated:` needs real playback on a device.
