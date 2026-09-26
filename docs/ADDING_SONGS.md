# Adding songs to the built-in songbook

Two files:
- [`assets/library.json`](../assets/library.json): the **full library**, every official karaoke upload of each label's
  channel, generated. Refresh it (new uploads, removed videos) with `python3 tools/harvest_library.py`: it reads channel
  metadata with `yt-dlp --flat-playlist` (nothing downloaded), parses titles, skips longplays/medleys/"with vocals"
  uploads, checks every video is embeddable, and keeps each video's keypad code. Add a label by adding a channel
  (URL, code range, intro skip, title parser) to `CHANNELS` in the tool. Then `cargo test --test library_test`.
- [`assets/catalog.json`](../assets/catalog.json): the **curated catalog**, hand-picked songs with a genre, measured
  intro and optionally a guide timing, compiled into the app. A video in the catalog is left out of the library.

The rest of this page is about the curated catalog. Requests come in through the **Song request** issue form; guide timings through the **Guide timing** form (the app's **Share on GitHub** button fills it in).

## Rules
- **Official uploads only**: the label's own karaoke channel (today: GMM Karaoke, Whattheduck, Muzik Move Karaoke). No fan re-uploads.
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
