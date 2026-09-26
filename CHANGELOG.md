# Changelog

User-visible changes per release. Live at https://ktv-rs.solana-thailand.workers.dev (Cloudflare Worker version in brackets).
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/) (0.x: anything may change).

## [0.15.0] - 2026-09-27
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
