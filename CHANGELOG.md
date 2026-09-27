# Changelog

User-visible changes per release. Live at https://ktv-rs.solana-thailand.workers.dev (Cloudflare Worker version in brackets).
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/) (0.x: anything may change).

## [Unreleased]
### Added
- Spoken MC (Settings → MC Voice: Off / ไทย / English). It announces each song as it goes on stage, with its own line for a ★ TIP request, and reads the tuning score when a take ends. Template lines (no LLM), three wordings per song, the same one each time for a given song. Only on-device voices are used: Chrome's network voices would send the text to Google, so with no local voice the MC stays silent. Off by default. It does not re-announce a replay or the song restored on load.

### Changed
- Search ranks results: a keypad code first, then titles that are the query, start with it or contain it, then artists, then romanised aliases. Before, matches came in songbook order.
- The request QR links `/request` directly: one fewer redirect on the guest's phone.
- Search finds "artist title" typed together (`ภูวศิษฐ์ รักไม่ไหว`): when the whole query is in no single field, every word must match somewhere in the song.

## [0.27.0] - 2026-09-28 (`0138c8af`)
### Added
- Request a song with a tip: a second, smaller QR beside the tip code opens a phone page (`/request.html`, plain HTML + JS, no wasm). The guest types the 5-digit song code, picks 0.5 / 1 / 2 / 5 USDC and pays in Phantom or Solflare. The booth finds the payment by the room's `reference`, and a request memo (`ktv:req:<code>`) with at least 0.10 USDC plays that song next, marked **★ TIP** in the queue and Up next. The room details ride in the link's `#fragment`, so no server logs them.
- Garland toast when a tip lands: "Garland for the singer! +1.50 USDC", or the requested song. It sits bottom right, never over the video, and closes after 8 s. Each on-chain payment is acted on once.
- Tips confirmed on-chain: while a song plays, the booth asks the Solana RPC for payments carrying that song's `reference` and shows "✓ Tip received: 1.50 USDC" under the QR (a screen-reader status line). A payment counts only if it succeeded and the singer's USDC balance went up; each is counted once. Devnet uses the free public RPC; Mainnet needs a Helius URL under Settings → Tip RPC, because the public mainnet RPC refuses browsers. Polls every 5 s and backs off to 60 s when offline or rate limited.

## [0.26.0] - 2026-09-28 (`39bae859`)
### Added
- Tip the singer: set a Solana wallet under Settings → Tip Wallet and a Solana Pay QR code shows under the player. Guests pay USDC from their phone wallet at an amount they choose. Each song gets a fresh `reference` and the memo `ktv:<code>`. Devnet (test USDC) by default, Mainnet is opt-in. The QR is built in Rust/wasm as one SVG path, uses level L error correction for a bigger module size, and sits beside the video, never over it.
- Privacy note and README: what a Solana tip makes public.

## [0.25.0] - 2026-09-28 (`b9024944`)
### Added
- If the original-vocal video stops following the song (an ad before it, or a stalled stream), the app switches to the karaoke audio after about 5 seconds and says so ("Original vocal is catching up"), then brings the original singer back as soon as it is in step again. Before, you could hear the ad, or silence, instead of the song.
### Changed
- Documentation links work on GitHub (they pointed at local file paths).
- CI: every job has `timeout-minutes: 25` and e2e runs with `--test-timeout=120000`, so a hung headless run now fails with logs instead of blocking for 6 hours.

## [0.24.0] - 2026-09-28 (`d0f7a8af`)
### Added
- **Vocal (original singer) for 2,415 more GMM Karaoke songs** (2,774 in all), timed automatically from the label's official audio track as in 0.23.0. Official original-vocal videos are also found for 3,130 more library songs, waiting to be lined up by ear in Guide Timing Tools (was 533).
- **Booth input**: Enter queues the song when the search box holds a 5-digit code (USB number pads); media keys ⏯ / ⏭; a standard game controller (A pause/play, B clear search, D-pad seek, Start next song).
- **Chords by ear** (in the practice row): type a chord at each change while the song plays, or tap one already used; shows the chord now and the next, with a display transpose (capo). Saved per song on this device; Copy chart exports it.
- **Party leaderboard**: name who sang on the result card; Queue → Party leaderboard ranks each singer's best take of the last 12 hours. Names stay on this device (privacy note updated).
### Changed
- The song-to-video guide list loads with the library after the first paint instead of inside the app bundle, so the app still starts as fast as before despite the ~7x bigger list.

## [0.23.0] - 2026-09-27 (`f6d03a90`)
### Added
- **Vocal (original singer) for 359 more songs**, timed automatically: for GMM Karaoke songs the app now uses the label's official audio track, which lines up at a fixed offset after GMM's 18-second intro (checked against the hand-timed songs: within 0.3 s for 11 of 12). Found by title and length only; no audio is analysed.
- With Vocal on, **Vocal ahead / Vocal behind** shift the singer by 0.5 s if a song is slightly off; the fix is saved on this device.

## [0.22.0] - 2026-09-27 (`6f7ad715`)
### Added
- Search forgives typos: when nothing matches as typed (or on the other keyboard layout), songs one typo away (two for longer queries) are listed, closest first, with a note saying so. "bodyslan" finds Bodyslam, "รักไม่ไวแล้ว" finds รักไม่ไหวแล้วโว้ย.

## [0.21.0] - 2026-09-27 (`393cd275`)
### Added
- Genres for ~2,100 library songs, taken from the label that publishes the song's official video: **Luk Thung** (GRAMMY GOLD, 1,367 songs), **Rock** (Genie Records, 512) and **Indie** (all of Smallroom, 250). The genre chips and Auto-DJ's "your usual genre" now include them. Songs from labels that mix genres stay without one rather than get a guess.
### Changed
- The weekly link check covers every video the app plays (9,459: curated, full library, official MVs), not only the 43 curated songs.

## [0.20.0] - 2026-09-27 (`30a9f845`)
### Added
- Musician tools under the time bar: a **practice loop** (A at the start of a part, B at its end: it replays A→B until ✕ or the next song) and **Chords ↗**, a web search for the song's chords in a new tab.
### Fixed
- The time under the player waits until the video has really started (it could run a second or two ahead while a slow video loaded) and settles on the right second when the browser blocks autoplay.

## [0.19.0] - 2026-09-27 (`6f016748`)
### Added
- **RS** (100 songs, keypad 60001+) and **Smallroom** (250 songs, 65001+) official karaoke playlists: 8,542 songs in all. Smallroom's English titles are searchable too.
- Official original-vocal videos found for 831 of the 1,000 most-sung songs (was 159 of 200), now also from GMM's GeneLab, White Music and We Records channels. As before, each one reaches singers once it is lined up by ear in Guide Timing Tools.

## [0.18.0] - 2026-09-27 (`0d73df0a`)
### Removed
- The key (♭/♯, KEY TRANSPOSE) buttons: they only changed a label, because a web page cannot change the pitch of the audio inside a YouTube player. (A browser extension such as Transpose can, on youtube.com.)
### Added
- Official original-vocal videos found for 159 of the 200 most-sung library songs. They get the **Vocal** button once they are lined up by ear: with Guide Timing Tools on, the timing panel offers the found video (**Use suggested MV**).
### Fixed
- A guide timed on this device for a library song now comes back every time the song is picked (it was lost when the song was queued again from the list or by Auto-DJ).

## [0.17.3] - 2026-09-27 (`f1913264`)
### Changed
- Faster start: preparing the full songbook for search takes ~30x less time (25 ms → 0.8 ms natively), which removes a brief freeze on slower phones a few seconds after the page opens.
### Fixed
- The highlighted **Play** button (shown when the browser blocked autoplay) had low-contrast white text; it is now dark on amber (WCAG AA).

## [0.17.2] - 2026-09-27 (`90bad3d8`)
### Added
- Link previews: sharing the site in LINE, Facebook, X or Discord shows a KTV-RS card with a Thai title and description. The search description is now in Thai and English.
### Changed
- Fullscreen search only works while the player is actually fullscreen (it no longer searches twice in the normal view).
### Security
- Every GitHub Action is pinned to an exact commit, so a moved tag cannot change what builds and deploys the site; Dependabot proposes updates weekly.

## [0.17.1] - 2026-09-27 (`6b2dd495`)
### Fixed
- On opening the site with autoplay blocked, the time under the player could still jump to 00:00; it now stays at the song's start until the video plays.

## [0.17.0] - 2026-09-27 (not deployed: its release check caught the fix above)
### Added
- Search in **Fullscreen**: type while the player is fullscreen and the matching songs list beside the video (top 8), each with **Play**, **Insert** and **Queue**; ✕ clears the search and keeps fullscreen.

## [0.16.2] - 2026-09-27 (`655085ed`)
### Fixed
- On opening the site, when the browser blocks the video from starting by itself (browsers do until you tap or press a key), the time under the player no longer runs on while the video stands still. After a moment the button shows **Play**: press it (or Space), or YouTube's play button, and the song starts with the time in step. The time also holds while a video is buffering.

## [0.16.1] - 2026-09-27 (`bc761993`)
### Changed
- The code is now licensed under the GNU AGPL v3.0 (it said MIT with no license file). Settings links the source code as AGPL-3.0.
- Settings credits Muzik Move alongside the other labels.
- Releases now deploy from GitHub Actions after the owner approves them.

## [0.16.0] - 2026-09-27 (`f0bbb862`)
### Added
- Auto-DJ now picks from the whole songbook, not only the curated songs: sing an artist through and their other songs come up next.
- 11 more GMM Karaoke songs whose titles were written slightly differently (8,189 songs in all).
### Fixed
- **Add URL** with a video that is already in the songbook queues that song (its own keypad code and intro skip) instead of adding a duplicate, and says so.
- About 160 library songs showed "Original Karaoke" or a stray bracket after the artist name, or a "(ซนซน 40 ปี GMM GRAMMY)" tag in the title; they read cleanly now and search better.
- A skipped library song no longer stops Auto-DJ from choosing any other library song.

## [0.15.0] - 2026-09-27 (`44dc6a23`)
### Added
- Full songbook: every official karaoke upload from **GMM Karaoke** (~7,900 songs), **Whattheduck** and **Muzik Move Karaoke**, ~8,200 songs in all. It loads in the background after the page opens; search, keypad codes, favourites and the queue all work with it. GMM's romanised titles are searchable ("kam ka sa ka la sin").
- The song list shows 60 songs at a time with **Show more** (type to narrow it down).

## [0.14.0] - 2026-09-27
### Added
- **TV** mode (header button): bigger text and player for a booth or TV screen, with **Up next** under the video.
### Fixed
- Desktop layout: player and songbook now sit side by side on one screen as designed (a broken CSS rule had stacked them since the first release). Phones are unchanged.

## [0.13.0] - 2026-09-27
### Added
- Install as an app (Add to Home Screen / Install): opens in its own window, handy for a karaoke booth or TV.
- New microphone app icon and favicon.

## [0.12.1] - 2026-09-27
### Fixed
- Songbook count says "1 Song", not "1 Songs".
### Added
- README screenshots, CI and prod-check badges; scheduled prod check (e2e against the live site every 6 hours).

## [0.12.0] - 2026-09-27 (`8b2f2f96`)
### Security
- Content-Security-Policy no longer allows `'unsafe-eval'`: the player-sync and keyboard scripts load as normal files and are called directly from Rust.

## [0.11.0] - 2026-09-27 (`84fb0e8d`)
### Added
- Privacy note (`/privacy.html`) listing everything stored on the device.
- Settings → **Clear my data on this device** (two taps).

## [0.10.0] - 2026-09-27 (`b7cf346a`)
### Fixed
- Pressing Space to pause after clicking a button (e.g. Next Song) no longer clicks that button again.
### Added
- Visible keyboard focus ring; a button reached with Tab takes Space/Enter.
- Reduced motion when the system asks for it; screen readers announce the song now singing.

## [0.9.0] - 2026-09-27 (`a9242b31`)
### Changed
- First paint on phones 4.3 s → 1.4 s (Lighthouse performance 54 → 88): stylesheet and fonts load with the page, loading screen while the app starts.
- Text contrast meets WCAG AA (Lighthouse accessibility 100); smaller favicon; `robots.txt`.

## [0.8.0] - 2026-09-27 (`99ac50e2`)
### Added
- Guide Timing Tools: **Share on GitHub** opens a pre-filled guide-timing issue; issue forms for guide timings and song requests.
### Changed
- 14% smaller app download (link-time optimisation).

## [0.7.0] - 2026-09-27 (`154394f1`)
### Added
- Thai-friendly search: tone marks and spaces optional, romanised titles (`rak mai wai`), and queries typed on the wrong keyboard layout are retried.
- ☆ Favourites and **Recently sung** shelves in the songbook.

## [0.6.0] - 2026-09-27 (`389e53dc`)
### Added
- Result card with the tuning score when a song ends; **Recent scores** in the Queue tab.
- Mic room check: the first second measures the room, quieter sound is ignored.
### Changed
- The Queue tab's End Song test button only shows with Guide Timing Tools on.

## [0.5.0] - 2026-09-27 (`c8d6a9b2`)
### Changed
- Queue and keyboard handling rewritten as tested modules; the unused BLAKE3 badge in Auto-DJ is gone.

## [0.4.0] - 2026-09-27 (`20981dfb`)
### Added
- Keyboard shortcut list (`?` or the header's **?** button), open on a first visit.
### Changed
- UI text in English throughout; song titles and artists marked as Thai for screen readers.

## [0.3.0] - 2026-09-27 (`4b6f9408`)
### Fixed
- Clicking inside a video no longer stops Type-to-Search.
- Phones and narrow windows: every player control stays reachable (they were cut off below ~900 px).
- Revert says what it does for songs added by URL.

## [0.2.0] - 2026-09-26 (`684e2638`)
### Added
- Guide Timing Tools: line up the original-vocal video by ear, save per device, copy JSON.
- Tuning score explanation on touch screens.
### Fixed
- Changing key no longer switches the vocal back to Karaoke.

## [0.1.0] - 2026-09-26 (`f1fb63c8`)
- First public release.

[0.14.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.12.1...v0.13.0
[0.12.1]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ozoneRatchapon/ktv-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ozoneRatchapon/ktv-rs/releases/tag/v0.1.0
