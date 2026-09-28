# Adding songs to the built-in songbook

Two files:
- [`assets/library.json`](../assets/library.json): the **full library**, every official karaoke upload of each label's
  channel, generated. Refresh it (new uploads, removed videos) with `python3 tools/harvest_library.py`: it reads channel
  metadata with `yt-dlp --flat-playlist` (nothing downloaded), parses titles, skips longplays/medleys/"with vocals"
  uploads, checks every video is embeddable, and keeps each video's keypad code. A code whose video leaves the library
  goes to [`tools/retired_codes.json`](../tools/retired_codes.json) and is never given to another song (commit it with
  the library). Add a label by adding a channel (URL, code range, intro skip, title parser) to `CHANNELS` in the tool.
  Then `python3 -m unittest discover -s tests -p "*_test.py"` (title parsing) and `cargo test --test library_test`.
- [`assets/catalog.json`](../assets/catalog.json): the **curated catalog**, hand-picked songs with a genre, measured
  intro and optionally a guide timing, compiled into the app. A video in the catalog is left out of the library.

The rest of this page is about the curated catalog. Requests come in through the **Song request** issue form; guide timings through the **Guide timing** form (the app's **Share on GitHub** button fills it in).

## Rules
- **Official uploads only**: the label's own karaoke channel (today: GMM Karaoke, Whattheduck, Muzik Move Karaoke, RS Music, Smallroom Karaoke, and LIT Entertainment's PiXXiE playlist). No fan re-uploads.
- **Nothing downloaded**: durations, intros and guide timings are read or judged from the embedded players (YouTube's terms forbid downloading or separating audio).

## Entry
```json
{
  "id": "gmm_034",
  "code": "10034",
  "title": "ชื่อเพลง",
  "artist": "ศิลปิน",
  "youtube_id": "11-char id of the karaoke video",
  "guide": { "video_id": "11-char id of the official MV", "offset_secs": -18.0, "rate": 1.0 },
  "duration_secs": 215,
  "intro_skip_secs": 18,
  "category": "Pop",
  "channel": "GMM Karaoke",
  "is_favorite": false
}
```
- `id`: `gmm_NNN` / `wtd_NNN`, unique. `code`: next free in the channel's series (GMM `100xx`, Whattheduck `200xx`); codes `90001`–`99999` are reserved for songs added by URL.
- `category`: one of `catalog::CATEGORIES` (Rock, Pop, Indie, Modern, Luk Thung, Classic 90s).
- `duration_secs`: the karaoke video's length as the player shows it. `intro_skip_secs`: where singing starts (GMM uploads open with a ~13 s bumper + silence, so usually 18).
- `guide` is optional. Time it in the app: Settings → **Guide Timing Tools**, then **Share on GitHub** or **Copy JSON**.

## Then
```bash
python3 tools/title_aliases.py --write   # romanised search aliases from the official titles
python3 tools/link_check.py              # every video still public + embeddable
cargo test --test catalog_test           # unique ids/codes, known category, valid ids, intro < duration
```

## Original-vocal video for library songs

Library songs get their **Vocal** button from [`assets/mv_guides.json`](../assets/mv_guides.json), keyed by the karaoke
video id. `python3 tools/match_mv.py [--top 200]` finds the official MV / lyric video for the most-watched GMM Karaoke
songs on the labels' channels (GMM GRAMMY OFFICIAL, Genierock, GRAMMY GOLD): metadata only, title **and** artist must
match, similar length, embeddable. It writes suggestions (`{"video_id": ...}`) and never touches a timed entry; every
match is listed in `tools/mv_candidates.json` (not committed) for review.

**Auto-timing (GMM Karaoke):** the same run looks for each song's **official audio track** on YouTube Music
(`tools/official_audio.py`). A GMM karaoke video is an 18 s intro plus the studio recording, and the audio track is that
recording, so it lines up at a fixed −18.2 s (checked against the hand-timed catalog: within 0.3 s for 11 of 12). Only
the first search result is taken, and only if the title matches, it is a music track (not live / cover / another
version) and it is 16–22 s shorter than the karaoke video. These entries carry `"auto": true`, reach singers at once,
and the player offers **Vocal ahead / Vocal behind** (±0.5 s, saved on the device) in case one is slightly off.
A hand-timed entry always wins over an automatic one.

Any other suggestion reaches singers only once it is **timed by ear** (the audio cannot be analysed without breaking YouTube's
terms):
1. Settings → **Guide Timing Tools** on, then play the song (search its code).
2. The timing panel shows **Official video found**: check it on YouTube, then **Use suggested MV**.
3. **Play guide**, **Hear both**, nudge until the singer and the karaoke line up (Mark in sync + Fit speed for drift), **Save**.
4. **Share on GitHub** (or Copy JSON), and add `"offset_secs"` and `"rate"` to the song's entry in `assets/mv_guides.json`.

Then `cargo test --test library_test` (entries must be library songs; timed ones reach the song, untimed ones do not).

The same run tags library songs with a **genre** where the label decides it (`GENRE_BY_CHANNEL` in the tool: GRAMMY
GOLD → Luk Thung, Genierock → Rock) and writes it to `assets/library.json` (`genres` per channel; a label-wide
`genre` such as Smallroom's Indie comes from `CHANNELS` in `tools/harvest_library.py`). A re-harvest keeps them.
