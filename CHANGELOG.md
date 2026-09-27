# Changelog

User-visible changes per release. Live at https://ktv-rs.solana-thailand.workers.dev (Cloudflare Worker version in brackets).
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/) (0.x: anything may change).

## [0.22.0] - 2026-09-27
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
