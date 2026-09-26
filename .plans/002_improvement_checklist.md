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
- [x] 20. wasm 968 KB, was 846 KB **(checked)** → find the growth, try `wasm-opt -Oz`. Done: `lto = true` + `codegen-units = 1` → wasm 1,031 → 882 KB (gzip 403 → 340 KB) at unchanged pitch speed; opt-level "s"/"z" gave no further size after dx's wasm-opt and were 2.2× slower (`bench/002_release_profile.md`). Growth since 846 KB came from features, not one dependency.
- [x] 21. Dead CSS in `main.css` (2,226 lines **(checked)**). Done: scripted check of all 241 class selectors against src/assets/index.html → only 6 unused icon rules, removed; search box lost the left padding kept for its long-gone icon.
- [x] 22. Lighthouse / Core Web Vitals on prod. Done (v0.9.0): Perf 54 → 88, A11y 95 → 100, SEO 91 → 100, FCP 4.3 → 1.4 s, LCP 4.9 → 2.2 s (`bench/003_lighthouse_prod.md`): stylesheet in the static head, boot splash, font preloads, 15 KB favicon, robots.txt, AA contrast.
- [x] 22b. CLS 0.208 on a first visit, attributed to the shortcut list. Root cause (Lighthouse trace): the shift is inside the YouTube player iframe (its poster collapsing when the player starts), weighted into page CLS; our main frame shifts 0.0001. Not ours to fix; documented in bench 003.
- [x] 19. Preload the next song's video for faster song changes. Decided against: it needs a hidden YouTube player, which the RMF/developer policies forbid (same reason the hidden guide was removed). The preconnect to youtube-nocookie.com (v0.9.0) is the compliant part.

## Wave 6 — accessibility
- [x] 30. Visible focus, keyboard navigation of song cards, contrast audit. Done: contrast (v0.9.0, Lighthouse a11y 100); one `:focus-visible` ring (scrubber had none); a control reached with Tab keeps Space/Enter, a mouse-clicked one does not. **Bug fixed on the way:** after clicking a button (e.g. Next Song), Space paused *and* re-clicked it — Chrome clicks on Space's keyup and turns on `:focus-visible` for a clicked button as soon as a key is pressed, so focus origin is now tracked (pointerdown → focusin) and the booth's Space also cancels its keyup. Tests: 7 Node, e2e keyboard + real mouse click.
- [x] 31. `prefers-reduced-motion`; announce song changes (`aria-live`). Done: reduced-motion media query (animations/transitions off); visually hidden `role=status` "Now singing: <title> by <artist>" (e2e-checked).

## Wave 7 — security & privacy
- [x] 23. Remove CSP `'unsafe-eval'` if Dioxus allows. Done: all 6 `document::eval` uses gone. `ktv_sync.js`/`ktv_keys.js` are classic scripts in the static `<head>` (`AssetOptions::js().with_static_head(true)`, hashed + immutable), so they run before the wasm; Rust installs them and calls methods through `Reflect` (`src/js_bridge.rs`, typed `JsArg`, messages on a futures channel); clipboard + fullscreen via web-sys (`src/browser.rs`). CSP now `script-src 'self' 'wasm-unsafe-eval' blob:`. Verified: e2e 16/16 under the strict CSP (fails on violations), `KtvSync.debug()` shows load mapping (exact −18.02), vocal switch, nudge, Space pause; fullscreen via real click.
- [x] 24. Versioned `localStorage` migrations (no data loss on schema change). Done as a guard, not a framework (no breaking change exists yet): `tests/storage_fixtures_test.rs` decodes values captured from the real app for all 5 keys (`tests/fixtures/storage/`), so a type change that would silently reset users' data fails CI; the test says to bump the key and migrate when that happens.
- [x] 25. Privacy page backing the "Zero Tracking" claim. Done: `public/privacy.html` (every stored key and why, mic stays local, YouTube/Cloudflare/GitHub roles), linked from Settings; **Clear my data on this device** (two taps) removes `ktv.*` + `_ktv_*` only and reloads (`storage::clear_all`, `is_app_key`). Tests: key predicate + e2e (other sites' keys untouched).

## Wave 8 — platform (Phase 3)
- [ ] 17. TV / booth mode (10-foot UI).
- [x] 32. PWA: installable, offline app shell. Done (installable part): `manifest.webmanifest` (standalone, theme/background colours), original mic icon (`public/icons/icon.svg` → 512/192/180 PNG + maskable; also replaces the Dioxus-template DNA favicon, 2 KB). Chrome reports no installability errors (normal context); e2e checks manifest + icons. No service worker: playback needs YouTube online anyway, and a SW cache risks serving stale app files after a release; revisit only if an offline songbook is wanted.
- [ ] 33. Phone remote. `gated:` Durable Objects blocked by Cloudflare `10013`/`10021`; recheck the versions API or pick another transport.

## Wave 9 — ops & docs
- [ ] 34. Deploy from CI with an approval environment. `gated:` see 34b (Cloudflare token as a GitHub secret).
- [x] 35. CHANGELOG / release notes per tag. Done: `CHANGELOG.md` (Keep a Changelog, v0.1.0–v0.12.0 with Worker version ids). Release flow now: add the entry, tag with `-m "vX.Y.Z: <summary>"`.
- [x] 36. Uptime check on the prod URL. Done: `.github/workflows/prod-check.yml` every 6 h — HTTP + CSP header + the full e2e suite against prod; opens/comments one `prod-check` issue on failure. First run 36270832447: 16/16.
- [x] 37. README screenshots + "adding songs" guide. Done: `docs/ADDING_SONGS.md`; `docs/screenshots/` (desktop standby + songbook, mobile romanised search, recent scores; standby so no video frames in the repo); README badges now real CI / prod-check status. Fixed "1 Songs".

## Gated (not worked until the owner/device says so)
- [ ] L1. No `LICENSE` file, but README shows an MIT badge linking to one (and `Cargo.toml` has no `license`). `gated:` owner picks the license and copyright holder.
- [ ] 34b. Deploy from CI (item 34). `gated:` needs a Cloudflare API token stored as a GitHub secret (owner) plus an approval environment.
- [ ] 1. Custom domain. `gated:` owner deferred (costs money).
- [ ] 3. Real mic test on iPhone Safari / Android Chrome. `gated:` needs devices.
- [ ] 4. YouTube pre-roll ads vs sync. `gated:` needs real playback on a device.
