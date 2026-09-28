# 004 — Karaoke medley (เมดเลย์)

Idea (owner, 2026-09-28): a live band plays medleys; karaoke rooms don't. Let the booth sing a chain of song parts
(e.g. chorus of A → verse+chorus of B → chorus of C) as one queue entry, with smooth transitions.

Prior art, as far as known (not verified): Japanese systems (DAM, JOYSOUND) ship ready-made medley *tracks* and a
"one chorus" mode. A medley the room builds itself from any songs in the library is the new part.

## What already exists (reuse, don't rebuild)
- A part of a song: practice loop A/B (`SyncCommand::SetLoop`, `components/practice.rs`), in karaoke seconds.
- Beats: tap tempo `BeatGrid` per song (`ktv.tempo.v1`, `src/tempo/`) and the count-in clicks.
- Key: chord charts per song (`ktv.chords.v1`), which can say which key a part is in.
- Volume ramping: `SyncCommand::SetVolume`, applied to both players.
- Arbitrary playback rate: the sync core already sets fractional rates on the guide player (`setPlaybackRate`).
- Auto next at the end of a video (v0.35.1) and the queue (`QueueItem`).
- Scores per take (`TakeResult`), party leaderboard, phone remote, share links (timing share).

## Hard limits (YouTube embeds)
- No audio access: the player is a cross-origin iframe, so no transposing, no real crossfade mixing through Web Audio.
  Transitions are volume ramps on two iframes; key change is impossible, so pick parts in compatible keys.
- Loading the next video takes time and may start with a pre-roll ad. The app must not hide or mute ads
  (YouTube terms). An ad at a join = the join becomes a short break; say so in the UI instead of pretending.
- Tempo matching by playback rate keeps pitch (browser default), but more than ~±6 % sounds wrong.

## Design
Data (`ktv.medleys.v1`, this device, shareable by link like timings):
```
Medley { id, title, parts: Vec<MedleyPart> }
MedleyPart { song_id, start: f64, end: f64 }   // karaoke seconds, end > start
```
Queue (as built in M1): each part is its own `QueueItem` with `part: Some(MedleySlot)` (title, n of m, span), back to
back, so auto next, skip, scores, telemetry and the session all work unchanged; the queue badges each part.

Player, M1: one deck; the sync core fades the part in and out and ends it at its end. The vocal switch stays usable
(the fades apply to the guide too). M2: two karaoke "decks" (A/B, like a DJ). While part N plays on one deck, part
N+1 waits cued on the other at its `start`. At `end − fade` the playing deck fades out while the other starts and
fades in, then decks swap. The guide MV is off during an M2 crossfade: a third iframe is too heavy for TV sticks.

## Phases
- [x] M1. Medley builder + hard join. Done 2026-09-29 (v0.37.0, Worker `c9a2b3ca`). `src/medley/` (pure): `Span` (validated, ≥ 5 s,
  inside the song), `guess_span` (B: after the intro skip or 15 s, 75 s long, capped at the song's end; labelled
  "guessed"), `MedleyBook { draft, saved, marked }` in `ktv.medleys.v1` (A: A/B + **+ Medley** in the practice row, or
  any nudge, is remembered per song and wins over the guess next time), 12 parts / 50 saved max, sanitized on load,
  `slots()` → `QueueItem.part: Option<MedleySlot>` (old sessions load). Booth: `add_medley` puts parts back to back;
  `Placement::Next` lands after the rest of a medley being sung (a TIP / Insert never splits one). Sync core:
  `set_part(end)`: silent until the player plays, 1 s fade-in, 2.5 s fade-out to the end, then one `ended` (the
  video's own end is not a second one); fades ride on the booth volume for both players. Player: starts at the part
  (embed `start=` whole second), banner "Medley 2/4 · title  00:18 → 01:33", no intro banner; Replay goes to the
  part's start. MC speaks before part 1 only; each part is its own take (score). UI: Queue tab → Medley under the
  queue list (add by code, nudge ±5 s, reorder, title, Play now / Queue / Save / Clear, saved list with Queue /
  Edit / ✕). Tests: `tests/medley_test.rs` (11), `ktv_sync.test.cjs` (+4), `sync_test`, e2e "medley" (add by code,
  unknown code, nudge → marked, save, Play now, part end → part 2, A/B → + Medley, reload keeps it).
  Found on the way: the same video from the same second (queued twice, or a medley repeating a part) reused the
  finished player (Dioxus honours `key` only in lists; only a changed `src` reloaded it). The karaoke iframe is now
  a keyed list of one per queue entry; e2e "same song twice" failed before and passes after.
  Join gap: measured ~0.5 s on prod (desktop Chrome, real YouTube: part 1 at 0 volume at its end → part 2 playing
  0.3 s after it loaded); expect more on a TV stick, and more again if YouTube plays an ad before part 2.
- [ ] M2. Two decks, gapless join: cue the next part early on the second karaoke iframe, overlap the fades.
  Needs a measure of TV-stick load (two decoding iframes) on a real device. Policy limit (YouTube RMF, as the guide
  pane already follows): a player may only play while visible (≥ 200×200), so deck B may wait loaded and paused
  but must be on screen before it plays: the crossfade shows both players side by side for its 2.5 s. An ad on
  deck B cannot be skipped or hidden, so the join falls back to M1 behaviour when one plays.
- [ ] M3. On the beat: if both songs have a `BeatGrid`, snap `end` and the next `start` to bar lines and start the
  next part so its first downbeat lands one bar after the last one; count-in clicks can bridge the join.
  Optional tempo match by rate when the BPMs are within ~6 %.
- [ ] M4. Key hint: when chord charts exist, show "same key / near key / far key" between neighbouring parts and
  suggest an order (circle of fifths). A hint only; nothing is transposed.
- [ ] M5. Room features: score per part + medley total on the result card; build / queue a medley from the phone
  remote; share a medley by link; a few ready-made medleys the host curates.
  - [x] M5a. Share by link. Done 2026-09-29 (v0.38.0, Worker `548e92f9`). `src/medley/share.rs` (pure): `share_fragment` writes
    `#medley=<title>&parts=<code>:<start>-<end>[g],…` (percent-encoded, times to 0.1 s, `g` = guessed, so the
    builder keeps saying "guessed"); `parse_fragment` drops unreadable parts and holds the builder's limits;
    `SharedMedley::open` looks codes up in the opener's songbook and skips (and counts) songs it lacks or times that
    do not fit. In the URL fragment, so it never reaches the server. The page opens on the Queue tab with an
    Open / Dismiss offer (a toast points at it), and the fragment is removed from the address bar at once so a
    reload does not offer it again. Share copies the link and also shows it (while the draft is unchanged) for
    copying by hand. `browser::copy_text` is now async and says whether the browser allowed the copy (it rejected
    silently before, as an unhandled promise, and three buttons claimed "Copied"). Tests: `medley_share_test` (6),
    e2e "medley share". Custom songs (Add URL) are skipped for the opener: their codes exist on one device only.
  - [x] M5b. Result card: the medley's total after its last part. Done 2026-09-29 (v0.39.0, Worker `04c89ab5`). `TakeResult.medley:
    Option<MedleyTake { id, index, count, title }>` (id = queue id of part 1: a medley's parts get consecutive ids),
    set by `PitchMeter` from `QueueItem::medley_take()` with the take's song. `score::medley_total` (pure): only on
    the last part's take; mean of the scored parts of that medley and voice (duet singers each get their own), a
    replayed part counts once with its latest take. The card's big number is the total, "Mean of n of m parts · last
    part x"; earlier parts' cards say "Medley i/m · title". The MC says the total after the last part (still quiet
    between parts). Plain mean, not note-weighted: easy to explain to a room. Tests: `medley_score_test` (5), e2e
    "medley score" (fake mic, two parts).
  - [ ] M5c. Phone remote: queue one of the booth's saved medleys (protocol: `PhoneCommand::QueueMedley`; the
    phone needs the list, so the booth publishes titles only).
  - [ ] M5d. Ready-made medleys the host curates (needs the owner's picks: owner-gated content).

## Decisions (owner, 2026-09-29)
1. Parts come from both A and B: the host's marked part (A/B or a nudge) wins; otherwise a guess, labelled as one.
2. Build now, following the recommended order and design (M1 first).
