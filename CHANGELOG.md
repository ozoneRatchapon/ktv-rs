# Changelog

User-visible changes per release. Live at https://ktv-rs.solana-thailand.workers.dev (Cloudflare Worker version in brackets).
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/) (0.x: anything may change).

## [0.41.0] - 2026-09-29 (`cc56e4fe`)
### Added
- Medley key hint: for songs with a chord chart entered on this device (Chords by ear), the medley builder shows each part's key, worked out from the chords inside the part, and whether each join moves to the same, a near or a far key (circle of fifths). When another order would join the keys more smoothly, **Order by key** reorders the parts, keeping the opener first where it can. It is a hint only: nothing is transposed.

## [0.40.0] - 2026-09-29 (`d1fca604`)
### Added
- Phone remote: the booth's saved medleys are listed on guests' phones (up to 12, title and number of parts), and one tap on **Queue** adds the whole medley to the end of the booth's queue, marked 📱 Phone. Like queuing a song, this works without the host allowing playback control.

## [0.39.0] - 2026-09-29 (`04c89ab5`)
### Added
- Medley total: with the mic on, the result card after a medley's last part shows the whole medley's score (the average of its parts that got a score, "Mean of 3 of 4 parts") and the spoken MC says it. Each earlier part's card says which part it was ("Medley 2/4 · title"). In a duet each singer gets their own total, and a part sung again (Replay) counts once, with its latest take.

## [0.38.0] - 2026-09-29 (`548e92f9`)
### Added
- Share a medley by link: **Share** in the medley builder copies a link (and shows it, for copying by hand). Whoever opens it lands on the Queue tab with the medley offered: **Open** puts it in their builder with the same songs, times and title, ready to play, save or nudge. Songs their songbook does not have are skipped, and the app says how many. The medley travels in the part of the address after `#`, which browsers never send to a server, and it leaves the address bar once read, so a reload does not offer it again.

### Fixed
- Copy buttons (Copy chart, the curator's Copy JSON) said "Copied" even when the browser refused the copy (it needs https). They now say when the copy was blocked.

## [0.37.1] - 2026-09-29 (`b4867817`)
### Fixed
- Pressing "Vocal ahead" / "Vocal behind" by mistake had no simple way back: the original timing could only be restored from the curator's Guide Timing Tools. A **Reset timing** button now sits next to them whenever a nudge is saved on this device; it goes back to the song's own timing at once and forgets the saved one.

## [0.37.0] - 2026-09-29 (`c9a2b3ca`)
### Added
- Medley: sing parts of several songs back to back as one queue entry (Queue tab → Medley). Add a song by its code and the app guesses its part (after the intro, about a verse and a chorus; nothing listens to the song, so it says "guessed"), then nudge the start and end 5 s at a time. Or, while a song plays, mark A and B in the practice row and press **+ Medley**: that part is used every time the song is added again. Save medleys on this device, play one now or queue it.
- Each part fades in from silence, fades out over its last 2.5 s and moves straight on to the next part. The player shows "Medley 2/4 · title" with the part's times, the queue marks each part, each part gets its own score, and the spoken MC talks only before the first part.
- A song requested "next" (Insert, ★ TIP) while a medley is being sung waits until the medley ends, so a medley is never split.

### Fixed
- The same song queued twice in a row did not start again from the top when its turn came: the app kept the first one's player instead of loading a new one. Every queue entry now gets a fresh player.

## [0.36.0] - 2026-09-28 (`18f43541`)
### Added
- Volume control: a slider in the player bar (0-100, saved on this device, 85 to start), ↑ / ↓ on the keyboard (5 a press) and a controller's D-pad up / down. It sets both the karaoke and the original-vocal players, stays the same for every next song, and switching Vocal no longer jumps it to full.
- Phone remote: 🔉 − / 🔊 + buttons (10 a tap) with the booth's volume shown, when the host lets phones control playback.

### Changed
- ↑ / ↓ no longer scroll the page (they change the volume); Page Up / Page Down and the mouse wheel still scroll the song list.

## [0.35.1] - 2026-09-28 (`7773e73e`)
### Fixed
- A song that played to its end did not move on to the next one (you had to press Next Song). YouTube only sends "state changed" events to a page that asks for them, and the app never asked, so it never heard that the video ended. The app now asks each karaoke player for them and also reads the end from the player's regular status reports, and one ending moves on exactly one song (a late second report of the same end no longer skips the song after it). Auto-DJ takes over at the end of the queue as before.

## [0.35.0] - 2026-09-28 (`ecfa7929`)
### Added
- Scrubb's official karaoke (8 songs: 72001-72007 from the band's channel, รอยยิ้ม 73001 from TERO MUSIC), under Indie. "Soulmate" finds คู่กัน.
- one31's drama songs with official karaoke (5 songs, 71001-71005: สองใจ, พิง, กลางหัวใจ, รักแท้, ฉันมันเป็นคนแบบนี้). The drama's name works as a search word ("วันทอง" finds สองใจ). These were every karaoke upload on one31's channel (its search and all 3,624 playlists); its other results are MVs and show clips.

## [0.34.0] - 2026-09-28 (`8f7a53b5`)
### Added
- PiXXiE's official karaoke (15 songs, codes 70001-70015) from the group's own channel and LIT Entertainment, under Pop. English titles work as search words ("deep talk" finds สนทนา), and the piano version of ติดฝน is listed as "(Piano Ver.)". The library refresh also added 6 new GMM Karaoke uploads.

## [0.33.1] - 2026-09-28 (`8eea12f8`)
### Changed
- With the spoken MC on, the mic ignores the moments the MC is talking, so its voice is never read or scored as singing (some systems play speech past the browser's echo cancellation).

## [0.33.0] - 2026-09-28 (`a793a873`)
### Added
- Duet (Settings → Duet): with a two-mic wireless receiver plugged in as one stereo device, the left mic is singer 1 and the right is singer 2. Each gets its own pitch readout, note lane, phrase and Tuning score (and Melody, where a song has melody data), its own result card and its own name for the party leaderboard. Echo cancellation is off in duet mode, because browsers merge it to one channel, so keep speakers away from the mics. Turn the mic on again after switching.

## [0.32.0] - 2026-09-28 (`7fcba75b`)
### Added
- Count-in for practising (musician tools row): tap **Tap** along with the beat four times or more and the song's tempo is saved on this device (♩ 96). **Count-in** then jumps to four beats before the loop's A (or before where the song is) and clicks them in, a higher click on the first, so the part starts on the beat. Taps that slip or skip a beat do not bend the tempo; a pause of two seconds starts the tapping over.

### Fixed
- Setting a loop's B jumped back to A rounded down to the whole second; it now jumps to A exactly.

## [0.31.0] - 2026-09-28 (`c5a20bec`)
### Added
- Melody score, for songs that have melody data on this device: how much of the tune you sang on the right note, in any octave (so low and high voices score alike), 100 at 80% of the tune's notes. It shows as **Melody** beside Tuning and on the result card. The note lane then also draws the tune (grey bars, moved to your octave) with 2 seconds of what is coming and a marker for now. No melody data ships yet: where it comes from is still being decided, so on every song today only Tuning shows.

### Changed
- The note lane follows the song's clock: it stands still while the video is paused and starts clean after seeking back or Replay.
- The mic reads the video clock through a lighter call (47 times a second, it no longer builds the whole sync snapshot).

### Fixed
- Phone remote server: a message to a phone or booth that was just disconnecting raised an error in the room (seen in CI logs); sends now skip closing connections.

## [0.30.0] - 2026-09-28 (`ed91d65e`)
### Added
- Phone remote (Keypad → Phone remote, off until the host turns it on). Guests scan the QR with a phone camera and get a page with the song on stage, the next five, song search and a 5-digit code box: *Add to the queue* puts the song at the back, marked 📱 Phone, and the phone is told what happened ("Queued 10001: …" or "No song with code …"). Skip, pause and replay buttons appear on phones only if the host ticks *Phones may skip, pause and replay*. The booth shows how many phones are connected; *New link* (two taps) makes a new code and sends phones on the old one away.
- How it works: the site now runs a small Cloudflare Worker with one Durable Object per room that relays commands and the song list; the booth holds the room's secret key on this device and the server stores no key, no names and no history (the last song list is kept a day, or until *New link*). Phones close their connection while the page is hidden. Limits: 32 phones per room, a few commands in a burst then one every 10 s.

### Changed
- The request page (`/request`) and the new remote page share one song search script (`/song_search.js`).
- Privacy page: the phone remote is the one feature that sends data through this site's server, and what it sends.

## [0.29.1] - 2026-09-28 (`1f92489e`)
### Fixed
- Phones: with the mic on, the Tuning score (and on narrow phones the note lane) ran off the right edge of the player and was cut off. The mic row now wraps inside the player at any width.

## [0.29.0] - 2026-09-28 (`a39a2669`)
### Added
- Note lane beside the mic readout: your held notes from the last 8 seconds on a semitone grid (a bar on a line is in tune, between lines is off; cyan / amber / red repeat it at a glance). It sits in the player bar, never over the video.
- Phrase score: after each breath (about half a second of quiet) the phrase you just sang gets its own tuning score, on the same 0-100 scale as the song. It needs two held notes. Like the song score it cannot know the melody, so it measures tuning, not the right notes.

## [0.28.0] - 2026-09-28 (`e1cf5a1f`)
### Added
- Phone request page search: type a title, an artist or a romanised name (Thai without tone marks works) and tap the song; no need to read its code off the booth. The song index (`/songs.txt`, ≈200 KB compressed, fetched only on first search) is built at release from the booth's own library, so codes always match, and it is ranked the same way as the booth search.
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
